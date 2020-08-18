use std::sync::{mpsc::Sender, Arc, RwLock};

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

use crate::renderers::renderer::{Command, Dimensions, Renderer};
use crate::PixelRendererCommunicator;
use raytracer_core::PixelColor;
use std::time::Instant;

struct Size {
    width: usize,
    height: usize,
}

pub struct RendererPixels {
    world: Arc<RwLock<World>>,
    tx: Sender<Command>,
}

impl Renderer for RendererPixels {
    fn new(dimensions: Dimensions, tx: Sender<Command>) -> Self {
        Self {
            world: Arc::new(RwLock::new(World::new(dimensions.height, dimensions.width))),
            tx,
        }
    }

    fn pixel_accessor(&mut self) -> PixelRendererCommunicator {
        PixelRendererCommunicator::new(Arc::clone(&self.world))
    }

    fn start_rendering(self) {
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
        let tx = self.tx;
        event_loop.run(move |event, _, control_flow| {
            let mut world = world_accessor.write().unwrap();
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
                if input.key_pressed(VirtualKeyCode::M) {
                    world.render_mode = match world.render_mode {
                        RenderMode::Normal => RenderMode::PerfTime,
                        RenderMode::PerfTime => RenderMode::Normal,
                    }
                }

                if input.key_pressed(VirtualKeyCode::Up) {
                    tx.send(Command::Up).unwrap();
                }
                if input.key_pressed(VirtualKeyCode::Down) {
                    tx.send(Command::Down).unwrap();
                }
                if input.key_pressed(VirtualKeyCode::Q) {
                    tx.send(Command::Left).unwrap();
                }
                if input.key_pressed(VirtualKeyCode::D) {
                    tx.send(Command::Right).unwrap();
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

enum RenderMode {
    Normal,
    PerfTime,
}

pub struct World {
    pixels: Vec<Pixel>,
    size: Size,
    max_write_count: u64,
    render_mode: RenderMode,
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
            max_write_count: 1,
            render_mode: RenderMode::Normal,
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
        if pixel.write_count > self.max_write_count {
            self.max_write_count = pixel.write_count;
        }
    }

    pub fn invalidate_pixels(&mut self) {
        let black = PixelColor { r: 0, g: 0, b: 0 };
        for pixel in &mut self.pixels {
            pixel.color = black;
            pixel.write_count = 0;
        }
    }

    /// Draw the `World` state to the frame buffer.
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, raw_pixel) in frame.chunks_exact_mut(4).enumerate().rev() {
            let x = (i % self.size.width as usize) as usize;
            let y = self.size.height - 1 - (i / self.size.width as usize) as usize;

            let pixel = &self.pixels[y * self.size.width + x];
            // Normal color mode:
            let rgba = match self.render_mode {
                RenderMode::Normal => [pixel.color.r, pixel.color.g, pixel.color.b, 0xff],
                RenderMode::PerfTime => [
                    ((pixel.write_count as f64 / self.max_write_count as f64) * 255.0) as u8,
                    0,
                    0,
                    0xff,
                ],
            };

            raw_pixel.copy_from_slice(&rgba);
        }
    }
}
