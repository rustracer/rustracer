#![feature(tau_constant)]
#![feature(clamp)]
#![feature(associated_type_bounds)]
#![feature(trait_alias)]

pub use nalgebra::Vector3;
use rand::seq::SliceRandom;

use crate::camera::Camera;
use crate::shapes::shape::Shape;

mod camera;
mod materials;
pub mod shapes;

#[derive(Debug, Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Vector3<f64>> for PixelColor {
    fn from(vector3: Vector3<f64>) -> Self {
        Self {
            r: vector3.x as u8,
            g: vector3.y as u8,
            b: vector3.z as u8,
        }
    }
}

pub struct PixelPosition {
    pub x: usize,
    pub y: usize,
}

pub type Scene<'a> = Vec<&'a dyn Shape>;
pub trait PixelRenderer = Fn(PixelPosition, PixelColor) + Send;

pub struct Raytracer {}

impl Raytracer {
    pub fn generate<T, R>(
        &self,
        width: f64,
        height: f64,
        scene: &[&dyn Shape],
        samples_per_pixel: i64,
        set_pixel: &T,
        random: &mut R,
    ) where
        T: PixelRenderer,
        R: rand::Rng + 'static + Send,
    {
        let camera = Camera::new();
        let random_positions = all_pixels_at_random(height as i64, width as i64, random);
        let scale = 1.0 / samples_per_pixel as f64;
        for pos in random_positions {
            let mut samples_color = Vector3::new(0.0, 0.0, 0.0);
            for _s in 0..samples_per_pixel {
                let offset_x = (pos.x as f64 + random.gen_range(0.0, 1.0)) / (width - 1.0);
                let offset_y = (pos.y as f64 + random.gen_range(0.0, 1.0)) / (height - 1.0);
                let r = camera.emit_ray_at(offset_x, offset_y);
                samples_color += r.project_ray(&scene);
            }
            let corrected_pixel_color = (samples_color * scale)
                .map(|c| c.clamp(0.0, 1.0))
                .map(f64::sqrt)
                .map(|c| c * 255.0);
            set_pixel(pos, PixelColor::from(corrected_pixel_color));
        }
    }
}

fn all_pixels_at_random<R>(height: i64, width: i64, rng: &mut R) -> Vec<PixelPosition>
where
    R: rand::Rng + 'static + Send,
{
    let mut random_y: Vec<i64> = (0..height).rev().collect();
    let mut random_x: Vec<i64> = (0..width).rev().collect();
    random_y.as_mut_slice().shuffle(rng);
    let mut random_positions: Vec<PixelPosition> = random_y
        .iter()
        .flat_map(|y| -> Vec<PixelPosition> {
            random_x.as_mut_slice().shuffle(rng);
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
    random_positions.as_mut_slice().shuffle(rng);
    random_positions
}
