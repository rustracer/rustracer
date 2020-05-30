use log::error;
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::sync::{Arc, Mutex};

use nalgebra::Vector3;

pub type Color = Vector3<f64>;

struct Size {
    width: usize,
    height: usize,
}

pub struct RendererPixels {
    world: Arc<Mutex<World>>,

}

impl RendererPixels {
    pub fn new(height: usize, width: usize, samples_per_pixel: i64) -> Self {

        let new = Self {
            world: Arc::new(Mutex::new(World::new(height, width, samples_per_pixel)))
        };
//        new.start_rendering();
        new
    }
    pub fn set_pixel(&mut self) -> Box<dyn Fn(usize, usize, Color)+Send> {
        let world_accessor = Arc::clone(&self.world);
        Box::new(move |x, y, color| {
            let mut world = world_accessor.lock().unwrap();
            world.set_pixel(x, y, color)
        })
    }
    pub fn render(&self) {
  
    }

    pub fn start_rendering(&mut self)
    {
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
            let surface_texture = SurfaceTexture::new(world.size.width as u32, world.size.height as u32, surface);
            Pixels::new(world.size.width as u32, world.size.height as u32, surface_texture).unwrap()
        };

        drop(world);
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
                world.update();
                window.request_redraw();
            }
        });
    }
}
/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    pixels: Vec<Color>,
    size: Size,
    samples_per_pixel: i64,
}
impl World {
    
    fn new(height: usize, width: usize, samples_per_pixel: i64) -> Self {
        let count = width * height;
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, || Vector3::new(0.0,0.0,0.0));
        Self {
            pixels: v,
            size: Size{width, height},
            samples_per_pixel,
        }
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y * self.size.width + x] = color;
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate().rev() {
            let x = (i % self.size.width as usize) as usize;
            let y = self.size.height - 1 - (i / self.size.width as usize) as usize;

            let color = self.pixels[y * self.size.width + x];
            let scale = 1.0 / self.samples_per_pixel as f64;
            let ir = (255.999 * (color[0] * scale).clamp(0.0, 1.0).sqrt()) as i64;
            let ig = (255.999 * (color[1] * scale).clamp(0.0, 1.0).sqrt()) as i64;
            let ib = (255.999 * (color[2] * scale).clamp(0.0, 1.0).sqrt()) as i64;
            
            let rgba = [ir as u8, ig as u8, ib as u8, 0xff];

            pixel.copy_from_slice(&rgba);
        }
    }
}