use std::sync::{Arc, RwLock};

use log::error;
use pixels::wgpu::Surface;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use raytracer_core::camera::Camera;
use raytracer_core::rendering::renderer::{Dimensions, Renderer, SceneContext};
use raytracer_core::{PixelColor, PixelPosition, Vector3};
use std::cell::RefCell;
use std::time::Instant;

struct Size {
    width: usize,
    height: usize,
}

pub struct MovableCamera {
    origin: Vector3<f64>,
    lower_left_corner: Vector3<f64>,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
}

impl MovableCamera {
    fn from(camera: Camera) -> Self {
        MovableCamera {
            origin: camera.origin,
            lower_left_corner: camera.lower_left_corner,
            horizontal: camera.horizontal,
            vertical: camera.vertical,
        }
    }

    fn to_immutable(&self) -> Camera {
        Camera {
            origin: self.origin.clone_owned(),
            lower_left_corner: self.lower_left_corner.clone_owned(),
            horizontal: self.horizontal.clone_owned(),
            vertical: self.vertical.clone_owned(),
        }
    }

    fn move_up(&mut self) {
        self.origin.y += 0.1;
    }
}

pub struct RendererPixels {
    world: Arc<RwLock<World>>,
    camera: Arc<RwLock<MovableCamera>>,
}

impl Renderer for RendererPixels {
    fn new(dimensions: Dimensions, camera: Camera) -> Self {
        Self {
            world: Arc::new(RwLock::new(World::new(dimensions.height, dimensions.width))),
            camera: Arc::new(RwLock::new(MovableCamera::from(camera))),
        }
    }

    fn pixel_accessor(&mut self) -> SceneContext {
        let world_accessor = Arc::clone(&self.world);
        let camera_accessor = Arc::clone(&self.camera);
        let renderer = Box::new(move |position: PixelPosition, color| {
            let mut world = world_accessor.write().unwrap();
            world.set_pixel(position.x, position.y, color)
        });
        let camera_accessor2 = Box::new(move || {
            let camera = camera_accessor.read().unwrap();
            camera.to_immutable()
        });
        SceneContext {
            pixel_renderer: renderer,
            camera_accessor: camera_accessor2,
        }
    }

    fn start_rendering(&mut self) {
        let world_accessor = Arc::clone(&self.world);
        let camera_accessor = Arc::clone(&self.camera);
        let world = world_accessor.read().unwrap();
        let mut input = WinitInputHelper::new();
        let event_loop = EventLoop::new();

        let window = {
            let size = LogicalSize::new(world.size.width as f64, world.size.height as f64);
            WindowBuilder::new()
                .with_title("Hello Pixels")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };
        let mut _hidpi_factor = window.scale_factor();
        let mut pixels = {
            let surface = Surface::create(&window);
            let surface_texture =
                SurfaceTexture::new(world.size.width as u32, world.size.height as u32, surface);
            Pixels::new(
                world.size.width as u32,
                world.size.height as u32,
                surface_texture,
            )
            .unwrap()
        };
        drop(world);
        let mut last_time = Instant::now();
        event_loop.run(move |event, _, control_flow| {
            let world = world_accessor.write().unwrap();
            // Draw the current frame
            if let Event::RedrawRequested(_) = event {
                world.draw(pixels.get_frame());
                if pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Handle input events
            if input.update(&event) {
                // Close events
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if input.key_pressed(VirtualKeyCode::Up) {
                    eprintln!("up!");
                    camera_accessor.write().unwrap().move_up();
                    world_accessor.write().unwrap().invalidate_pixels();
                    return;
                }

                // Adjust high DPI factor
                if let Some(factor) = input.scale_factor_changed() {
                    _hidpi_factor = factor;
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    pixels.resize(size.width, size.height);
                }

                // dynamic time step from : https://gameprogrammingpatterns.com/game-loop.html
                let elapsed = last_time.elapsed().as_secs_f32();
                if elapsed > 1.0 / 10.0 {
                    last_time = Instant::now();
                    window.request_redraw();
                }
            }
        });
    }
}

struct Pixel {
    color: PixelColor,
    write_count: u64,
}

struct World {
    pixels: Vec<Pixel>,
    size: Size,
}

impl World {
    fn new(height: usize, width: usize) -> Self {
        let count = width * height;
        let mut pixels = Vec::with_capacity(count);
        let black = PixelColor { r: 0, g: 0, b: 0 };
        pixels.resize_with(count, || Pixel {
            color: black,
            write_count: 0,
        });
        Self {
            pixels,
            size: Size { width, height },
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, new_pixel: PixelColor) {
        // NOTE: this is not thread safe
        let mut pixel = &mut self.pixels[y * self.size.width + x];
        if pixel.write_count == 0 {
            pixel.color = new_pixel;
            pixel.write_count += 1;
            return;
        }
        let new_weight = 1.0 - pixel.write_count as f32 / (pixel.write_count + 1) as f32;
        let old_weight = 1.0 - new_weight;

        pixel.color.r = (pixel.color.r as f32 * old_weight + new_pixel.r as f32 * new_weight) as u8;
        pixel.color.g = (pixel.color.g as f32 * old_weight + new_pixel.g as f32 * new_weight) as u8;
        pixel.color.b = (pixel.color.b as f32 * old_weight + new_pixel.b as f32 * new_weight) as u8;
        pixel.write_count += 1;
    }

    /// Draw the `World` state to the frame buffer.
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, raw_pixel) in frame.chunks_exact_mut(4).enumerate().rev() {
            let x = (i % self.size.width as usize) as usize;
            let y = self.size.height - 1 - (i / self.size.width as usize) as usize;

            let pixel = &self.pixels[y * self.size.width + x];
            let rgba = [pixel.color.r, pixel.color.g, pixel.color.b, 0xff];

            raw_pixel.copy_from_slice(&rgba);
        }
    }

    fn invalidate_pixels(&mut self) {
        for idx in 0..self.pixels.len() {
            self.pixels[idx].write_count = 0;
        }
    }
}
