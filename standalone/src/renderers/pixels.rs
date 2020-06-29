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

use crate::renderers::renderer::{Dimensions, Renderer};
use raytracer_core::{PixelAccessor, PixelColor};
use std::time::Instant;

struct Size {
    width: usize,
    height: usize,
}

pub struct RendererPixels {
    world: Arc<RwLock<World>>,
}

impl Renderer for RendererPixels {
    fn new(dimensions: Dimensions) -> Self {
        let new = Self {
            world: Arc::new(RwLock::new(World::new(dimensions.height, dimensions.width))),
        };
        //        new.start_rendering();
        new
    }

    fn pixel_accessor(&mut self, weight: f32) -> Box<PixelAccessor> {
        let world_accessor = Arc::clone(&self.world);
        Box::new(move |position, color| {
            let mut world = world_accessor.write().unwrap();
            world.set_pixel(position.x, position.y, color, weight)
        })
    }

    // fn render(&self) {}

    fn start_rendering(&mut self) {
        let world_accessor = Arc::clone(&self.world);
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

struct World {
    pixels: Vec<PixelColor>,
    size: Size,
}

impl World {
    fn new(height: usize, width: usize) -> Self {
        let count = width * height;
        let mut v = Vec::with_capacity(count);
        let black = PixelColor { r: 0, g: 0, b: 0 };
        v.resize_with(count, || black);
        Self {
            pixels: v,
            size: Size { width, height },
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: PixelColor, weight: f32) {
        if (weight == 1.0) {
            self.pixels[y * self.size.width + x] = color;
            return;
        }
        // NOTE: this is not thread safe
        let existing_weight = 1.0 - weight;
        let mut existing_pixel = self.pixels[y * self.size.width + x];
        existing_pixel.r = (existing_pixel.r as f32 * existing_weight) as u8 + (color.r as f32 * weight) as u8;
        existing_pixel.g = (existing_pixel.g as f32 * existing_weight) as u8 + (color.g as f32 * weight) as u8;;
        existing_pixel.b = (existing_pixel.b as f32 * existing_weight) as u8 + (color.b as f32 * weight) as u8;;
        self.pixels[y * self.size.width + x] = existing_pixel;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate().rev() {
            let x = (i % self.size.width as usize) as usize;
            let y = self.size.height - 1 - (i / self.size.width as usize) as usize;

            let color = self.pixels[y * self.size.width + x];
            let rgba = [color.r, color.g, color.b, 0xff];

            pixel.copy_from_slice(&rgba);
        }
    }
}
