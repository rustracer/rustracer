use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};

use log::error;
use pixels::wgpu::Surface;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::rand_range_f64::rand_range_f64;
use crate::renderers::renderer::{Dimensions, PixelAccessor, PixelColor, Renderer};

struct Size {
    width: usize,
    height: usize,
}

pub struct RendererPixels {
    world: Arc<Mutex<World>>,
}

impl Renderer for RendererPixels {
    fn new(dimensions: Dimensions) -> Self {
        let new = Self {
            world: Arc::new(Mutex::new(World::new(dimensions.height, dimensions.width))),
        };
        //        new.start_rendering();
        new
    }

    fn pixel_accessor(&mut self) -> Box<PixelAccessor> {
        let world_accessor = Arc::clone(&self.world);
        Box::new(move |position, color| {
            let mut world = world_accessor.lock().unwrap();
            world.set_pixel(position.x, position.y, color)
        })
    }

    // fn render(&self) {}

    fn start_rendering(&mut self) {
        let world_accessor = Arc::clone(&self.world);
        let world = world_accessor.lock().unwrap();
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
        let mut hidpi_factor = window.scale_factor();
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
        drop(window);
        event_loop.run(move |event, _, control_flow| {
            let mut world = world_accessor.lock().unwrap();
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
            if input.update(event) {
                // Close events
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                // Adjust high DPI factor
                if let Some(factor) = input.scale_factor_changed() {
                    hidpi_factor = factor;
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    pixels.resize(size.width, size.height);
                }

                // Update internal state and request a redraw
                if rand_range_f64(0.0, 1.0) < 0.001 {
                    world.update();
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

    pub fn set_pixel(&mut self, x: usize, y: usize, color: PixelColor) {
        self.pixels[y * self.size.width + x] = color;
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {}

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
