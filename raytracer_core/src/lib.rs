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
    pub status: GenerationStatus,
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
            status: GenerationStatus::Unstable,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PixelPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenerationStatus {
    Unstable,
    Final,
}

pub struct PixelCache {
    pub pos: PixelPosition,
    pub last_color: Option<PixelColor>,
    pub incremental_raw_light: Option<Vector3<f64>>,
    pub status: GenerationStatus,
    pub same_color_count: u8,
    pub nb_samples: u64,
}

pub type Scene<'a> = Vec<&'a dyn Shape>;

pub trait PixelRenderer {
    fn set_pixel(&mut self, pos: PixelPosition, color: PixelColor);
    fn invalidate_pixels(&mut self);
}

pub struct GeneratorData {
    pub full_screen_render_count: u64,
    pub index: usize,
    pub pixel_cache: Vec<PixelCache>,
}

pub struct RandomGenerator {
    data: GeneratorData,
}

impl RandomGenerator
{
    pub fn new<R>(width: i64, height: i64, random: &mut R) -> RandomGenerator
    where
        R: rand::Rng + 'static + Send {
            RandomGenerator {
                data: GeneratorData {
                    full_screen_render_count: 0,
                    index: 0,
                    pixel_cache: all_pixels_at_random(height, width, random)
                }
            }
    }
    pub fn invalidate_pixels<R>(&mut self, width: i64, height: i64, random: &mut R)
    where
        R: rand::Rng + 'static + Send {
        let random_positions = all_pixels_at_random(
            height as i64,
            width as i64,
            random,
        );
        self.data.index = 0;
        self.data.pixel_cache = random_positions;
        self.data.full_screen_render_count = 0;
    }
}

impl GeneratorProgress for RandomGenerator {
    fn get_pixel(&mut self) -> &mut PixelCache
    {
        return &mut self.data.pixel_cache[self.data.index];
    }
    fn next(&mut self) -> Option<()> {
        self.data.index = self.data.index + 1;
        if self.data.index == self.data.pixel_cache.len() {
            self.data.index = 0;
            self.data.full_screen_render_count = self.data.full_screen_render_count + 1;
            None
        }
        else {
            Some(())
        }
    }
    fn get_index(&self) -> (u64, usize) {
        (self.data.full_screen_render_count, self.data.index)
    }
}

pub trait GeneratorProgress
{
    fn get_pixel(&mut self) -> &mut PixelCache;
    fn next(&mut self) -> Option<()>;
    fn get_index(&self) -> (u64, usize);
}

struct RaytracerInfo<R>
where
    R: rand::Rng + 'static + Send,
{
    pub width: f64,
    pub height: f64,
    pub random: R,
}
pub struct Raytracer<R>
where
    R: rand::Rng + 'static + Send,
{
    pub camera: Camera,
    info: RaytracerInfo<R>,
}

const MAX_SIMILAR_SAMPLE_FOR_A_PIXEL: u8 = 3;

impl<R> Raytracer<R>
where
    R: rand::Rng + 'static + Send,
{
    pub fn new(width: f64, height: f64, random: R) -> Self {

        Raytracer {
            camera: Camera::new(-1.8_f64, 1_f64, 2_f64),
            info: RaytracerInfo {
                width,
                height,
                random,
            },
        }
    }

    pub fn generate_pixel<S: PixelRenderer, G: GeneratorProgress>(
        &mut self,
        generator: &mut G,
        scene: &[&dyn Shape],
        samples: u64,
        renderer: &mut S,
    ) {
        let mut pixel = &mut generator.get_pixel();
        if pixel.status == GenerationStatus::Final {
            return;
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
        let mut color = PixelColor::from(corrected_pixel_color);
        if let Some(last_color) = pixel.last_color {
            if last_color == color {
                pixel.same_color_count += 1;
            } else {
                pixel.same_color_count = 0;
            }
        }
        if pixel.same_color_count > MAX_SIMILAR_SAMPLE_FOR_A_PIXEL {
            pixel.status = GenerationStatus::Final;
        }
        color.status = pixel.status;
        pixel.last_color = Some(color);
        renderer.set_pixel(pixel.pos, color);
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
                status: GenerationStatus::Unstable,
                nb_samples: 0,
                incremental_raw_light: None,
            }
        })
        .collect();
    random_positions.as_mut_slice().shuffle(rng);
    random_positions
}
