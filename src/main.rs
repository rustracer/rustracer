#![feature(tau_constant)]
#![feature(clamp)]

use std::thread;

use nalgebra::Vector3;
use rand::prelude::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::camera::Camera;
use crate::renderers::pixels::RendererPixels;
use crate::renderers::renderer::{Dimensions, PixelColor, PixelPosition, Renderer};
use crate::shapes::shape::Shape;
use crate::shapes::sphere::Sphere;
use std::borrow::BorrowMut;

mod camera;
mod materials;
mod renderers;
mod shapes;

const SAMPLES_PER_PIXEL: i64 = 10;

fn main_loop() {
    let camera = Camera::new();
    let width = 1920.0;
    let height = 1080.0;

    let mut renderer = RendererPixels::new(Dimensions {
        height: height as usize,
        width: width as usize,
    });

    let set_pixel = renderer.pixel_accessor();
    eprint!("Scanlines remaining:\n");
    thread::spawn(move || {
        let mut rng = SmallRng::from_entropy();

        let sphere = Sphere::new_with_metal(Vector3::new(0.0, 0.0, -1.0), 0.5);
        let sphere2 = Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0);
        let sphere3 = Sphere::new(Vector3::new(0.5, -0.4, -0.85), 0.1);
        let scene: Vec<&dyn Shape> = vec![&sphere, &sphere2, &sphere3];

        let scale = 1.0 / SAMPLES_PER_PIXEL as f64;

        let random_positions = all_pixels_at_random(height as i64, width as i64);

        for pos in random_positions {
            let mut samples_color = Vector3::new(0.0, 0.0, 0.0);
            for _s in 0..SAMPLES_PER_PIXEL {
                let offset_x = (pos.x as f64 + rng.gen_range(0.0, 1.0)) / (width - 1.0);
                let offset_y = (pos.y as f64 + rng.gen_range(0.0, 1.0)) / (height - 1.0);
                let r = camera.emit_ray_at(offset_x, offset_y);
                samples_color += r.project_ray(&scene);
            }

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
    let mut rng = SmallRng::from_entropy();

    let mut random_y: Vec<i64> = (0..height).rev().collect();
    let mut random_x: Vec<i64> = (0..width).rev().collect();
    random_y.as_mut_slice().shuffle(rng.borrow_mut());
    let mut random_positions: Vec<PixelPosition> = random_y
        .iter()
        .flat_map(|y| -> Vec<PixelPosition> {
            random_x.as_mut_slice().shuffle(rng.borrow_mut());
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
    random_positions.as_mut_slice().shuffle(rng.borrow_mut());
    random_positions
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
