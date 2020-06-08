#![feature(tau_constant)]
#![feature(clamp)]

use std::thread;

use nalgebra::Vector3;

use crate::camera::Camera;
use crate::rand_range_f64::rand_range_f64;
use crate::rand_range_f64::shuffle;
use crate::renderers::pixels::RendererPixels;
use crate::renderers::renderer::{Dimensions, PixelColor, PixelPosition, Renderer};
use crate::shapes::shape::Shape;
use crate::shapes::sphere::Sphere;

mod camera;
mod materials;
mod rand_range_f64;
mod renderers;
mod shapes;

const SAMPLES_PER_PIXEL: i64 = 4;

fn main_loop() {
    let camera = Camera::new();
    let width = 640.0;
    let height = 360.0;

    let mut renderer = RendererPixels::new(Dimensions {
        height: height as usize,
        width: width as usize,
    });

    let mut set_pixel = renderer.pixel_accessor();
    eprint!("Scanlines remaining:\n");
    let handle = thread::spawn(move || {
        let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5);
        let sphere2 = Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0);
        let sphere3 = Sphere::new(Vector3::new(0.5, -0.4, -0.85), 0.1);
        let scene: Vec<&dyn Shape> = vec![&sphere, &sphere2, &sphere3];

        let scale = 1.0 / SAMPLES_PER_PIXEL as f64;

        let random_positions = all_pixels_at_random(height as i64, width as i64);

        for pos in random_positions {
            let mut samples_color = Vector3::new(0.0, 0.0, 0.0);
            for _s in 0..SAMPLES_PER_PIXEL {
                let offset_x = (pos.x as f64 + rand_range_f64(0.0, 1.0)) / (width - 1.0);
                let offset_y = (pos.y as f64 + rand_range_f64(0.0, 1.0)) / (height - 1.0);
                let r = camera.emit_ray_at(offset_x, offset_y);
                samples_color += r.project_ray(&scene);
            }
            /*
            <<<<<<< HEAD
                        let pixel_color = PixelColor {
                            r: ((samples_color.x * scale).clamp(0.0, 1.0).sqrt() * 255.0) as u8,
                            g: ((samples_color.y * scale).clamp(0.0, 1.0).sqrt() * 255.0) as u8,
                            b: ((samples_color.z * scale).clamp(0.0, 1.0).sqrt() * 255.0) as u8,
                        };
                        set_pixel(pos, pixel_color);
                    }
                });
            =======
            */

            let scale = 1.0 / SAMPLES_PER_PIXEL as f64;
            let corrected_pixel_color = (samples_color * scale)
                .map(|c| c.clamp(0.0, 1.0))
                .map(f64::sqrt)
                .map(|c| c * 255.0);
            set_pixel(pos, PixelColor::from(corrected_pixel_color));
        }
    });
    renderer.start_rendering();

    eprint!("\nDone! :-)\n");
}

fn all_pixels_at_random(height: i64, width: i64) -> Vec<PixelPosition> {
    let mut random_y: Vec<i64> = (0..height).rev().collect();
    let mut random_x: Vec<i64> = (0..width).rev().collect();
    shuffle(random_y.as_mut_slice());
    let mut random_positions: Vec<PixelPosition> = random_y
        .iter()
        .flat_map(|y| -> Vec<PixelPosition> {
            shuffle(random_x.as_mut_slice());
            random_x
                .iter()
                .map(|x| -> PixelPosition {
                    PixelPosition {
                        y: *y as usize,
                        x: *x as usize,
                    }
                })
                .collect()
        })
        .collect();
    shuffle(random_positions.as_mut_slice());
    random_positions
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
