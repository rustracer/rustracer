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

use crate::renderers::renderer::{Command, Dimensions, MoveCommand, Renderer, RotateCommand};
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
                        RenderMode::PerfTime => RenderMode::Status,
                        RenderMode::Status => RenderMode::Normal,
                    }
                }

                let elapsed = last_time.elapsed().as_secs_f32();
                // dynamic time step from : https://gameprogrammingpatterns.com/game-loop.html
                if elapsed > 1.0 / 10.0 {
                    if input.key_held(VirtualKeyCode::Z) || input.key_held(VirtualKeyCode::Up) {
                        tx.send(Command::Move(MoveCommand {
                            x: 0.0,
                            z: 1_f64 * elapsed as f64,
                            y: 0.0,
                        }))
                        .unwrap();
                    }
                    if input.key_held(VirtualKeyCode::S) || input.key_held(VirtualKeyCode::Down) {
                        tx.send(Command::Move(MoveCommand {
                            x: 0.0,
                            z: -1_f64 * elapsed as f64,
                            y: 0.0,
                        }))
                        .unwrap();
                    }
                    if input.key_held(VirtualKeyCode::Q) || input.key_held(VirtualKeyCode::Left) {
                        tx.send(Command::Move(MoveCommand {
                            x: -1_f64 * elapsed as f64,
                            z: 0.0,
                            y: 0.0,
                        }))
                        .unwrap();
                    }
                    if input.key_held(VirtualKeyCode::D) || input.key_held(VirtualKeyCode::Right) {
                        tx.send(Command::Move(MoveCommand {
                            x: 1_f64 * elapsed as f64,
                            z: 0.0,
                            y: 0.0,
                        }))
                        .unwrap();
                    }
                    if input.key_held(VirtualKeyCode::A) {
                        // TODO: send rotate
                        tx.send(Command::Rotate(RotateCommand {
                            x: 0.0,
                            z: 0.0,
                            y: 1_f64 * elapsed as f64,
                        }))
                        .unwrap();
                    }
                    if input.key_held(VirtualKeyCode::E) {
                        // TODO: send rotate
                        tx.send(Command::Rotate(RotateCommand {
                            x: 0.0,
                            z: 0.0,
                            y: -1_f64 * elapsed as f64,
                        }))
                        .unwrap();
                    }
                    // Adjust high DPI factor
                    if let Some(factor) = input.scale_factor_changed() {
                        _hidpi_factor = factor;
                    }

                    // Resize the window
                    if let Some(size) = input.window_resized() {
                        pixels.resize(size.width, size.height);
                    }
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
    Status,
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
        let black = PixelColor {
            r: 0,
            g: 0,
            b: 0,
            status: raytracer_core::GenerationStatus::Unstable,
        };
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
        pixel.color = new_pixel;
        pixel.write_count += 1;
        if pixel.write_count > self.max_write_count {
            self.max_write_count = pixel.write_count;
        }
    }

    pub fn invalidate_pixels(&mut self) {
        let black = PixelColor {
            r: 0,
            g: 0,
            b: 0,
            status: raytracer_core::GenerationStatus::Unstable,
        };
        for pixel in &mut self.pixels {
            pixel.color = black;
            pixel.write_count = 0;
        }
        self.max_write_count = 0;
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
                RenderMode::PerfTime => {
                    let ratio = pixel.write_count as f64 / self.max_write_count as f64;
                    [(ratio * 255.0) as u8, 0, 0, 0xff]
                }
                RenderMode::Status => {
                    let isDone = pixel.color.status == raytracer_core::GenerationStatus::Final;
                    [0, if isDone { 255 } else { 0 }, 0, 0xff]
                }
            };

            raw_pixel.copy_from_slice(&rgba);
        }
    }
}
