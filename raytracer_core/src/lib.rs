#![feature(tau_constant)]
#![feature(clamp)]
#![feature(associated_type_bounds)]
#![feature(trait_alias)]

pub use nalgebra::Vector3;
use rand::seq::SliceRandom;

use crate::camera::Camera;
use crate::shapes::shape::Shape;

pub mod camera;
pub mod materials;
pub mod shapes;

#[derive(Debug, Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PartialEq for PixelColor {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
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

#[derive(Debug, Clone, Copy)]
pub struct PixelPosition {
    pub x: usize,
    pub y: usize,
}

pub struct PixelCache {
    pub pos: PixelPosition,
    pub last_color: Option<PixelColor>,
    pub incremental_raw_light: Option<Vector3<f64>>,
    pub same_color_count: u8,
    pub nb_samples: u64,
}

pub type Scene<'a> = Vec<&'a dyn Shape>;

pub trait PixelRenderer {
    fn set_pixel(&mut self, pos: PixelPosition, color: PixelColor);
    fn invalidate_pixels(&mut self);
}

pub struct Generator {
    pub index: usize,
}

struct RaytracerInfo<R, S>
where
    R: rand::Rng + 'static + Send,
    S: PixelRenderer,
{
    pub width: f64,
    pub height: f64,
    pub random: R,
    pub renderer: S,
}
pub struct Raytracer<R, S>
where
    R: rand::Rng + 'static + Send,
    S: PixelRenderer,
{
    pub camera: Camera,
    pixel_cache: Vec<PixelCache>,
    info: RaytracerInfo<R, S>,
}

const MAX_SIMILAR_SAMPLE_FOR_A_PIXEL: u8 = 3;

impl<R, S> Raytracer<R, S>
where
    R: rand::Rng + 'static + Send,
    S: PixelRenderer,
{
    pub fn new(width: f64, height: f64, mut random: R, renderer: S) -> Self {
        let random_positions = all_pixels_at_random(height as i64, width as i64, &mut random);

        Raytracer {
            pixel_cache: random_positions,
            camera: Camera::new(-1.8_f64, 1_f64, 2_f64),
            info: RaytracerInfo {
                width,
                height,
                random,
                renderer,
            },
        }
    }

    pub fn get_new_generator(&self) -> Generator {
        return Generator { index: 0 };
    }

    pub fn generate_pixel(
        &mut self,
        generator: &mut Generator,
        scene: &[&dyn Shape],
        samples: u64,
    ) -> Option<()> {
        if generator.index >= self.pixel_cache.len() {
            return None;
        }
        let mut pixel = &mut self.pixel_cache[generator.index];
        if pixel.same_color_count > MAX_SIMILAR_SAMPLE_FOR_A_PIXEL {
            return Some(());
        }
        let mut samples_color = Vector3::new(0.0, 0.0, 0.0);
        for _s in 0..samples {
            let offset_x = (pixel.pos.x as f64 + self.info.random.gen_range(0.0, 1.0))
                / (self.info.width - 1.0);
            let offset_y = (pixel.pos.y as f64 + self.info.random.gen_range(0.0, 1.0))
                / (self.info.height - 1.0);
            let r = self.camera.emit_ray_at(offset_x, offset_y);
            samples_color += r.project_ray(&scene);
        }
        if let Some(incremental_raw_light) = pixel.incremental_raw_light {
            samples_color = samples_color + incremental_raw_light;
        }
        pixel.incremental_raw_light = Some(samples_color);
        pixel.nb_samples = pixel.nb_samples + samples;
        let scale = 1.0 / (pixel.nb_samples) as f64;
        let corrected_pixel_color = (samples_color * scale)
            .map(|c| c.clamp(0.0, 1.0))
            .map(f64::sqrt)
            .map(|c| c * 255.0);
        let color = PixelColor::from(corrected_pixel_color);
        self.info.renderer.set_pixel(pixel.pos, color);

        if let Some(last_color) = pixel.last_color {
            if last_color == color {
                pixel.same_color_count += 1;
            } else {
                pixel.same_color_count = 0;
            }
        }
        pixel.last_color = Some(color);
        Some(())
    }

    pub fn generate(&mut self, scene: &[&dyn Shape], samples_per_pixel: u64) {
        let mut generator = self.get_new_generator();
        while self
            .generate_pixel(&mut generator, scene, samples_per_pixel)
            .is_some()
        {}
    }

    pub fn invalidate_pixels(&mut self) {
        let random_positions = all_pixels_at_random(
            self.info.height as i64,
            self.info.width as i64,
            &mut self.info.random,
        );
        self.pixel_cache = random_positions;
        self.info.renderer.invalidate_pixels();
    }
}

fn all_pixels_at_random<R>(height: i64, width: i64, rng: &mut R) -> Vec<PixelCache>
where
    R: rand::Rng + 'static + Send,
{
    let mut random_y: Vec<i64> = (0..height).rev().collect();
    let mut random_x: Vec<i64> = (0..width).rev().collect();
    random_y.as_mut_slice().shuffle(rng);
    let mut random_positions: Vec<PixelCache> = random_y
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
        .map(|pix| -> PixelCache {
            PixelCache {
                last_color: None,
                pos: pix,
                same_color_count: 0,
                nb_samples: 0,
                incremental_raw_light: None,
            }
        })
        .collect();
    random_positions.as_mut_slice().shuffle(rng);
    random_positions
}
