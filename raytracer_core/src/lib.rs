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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy)]
pub struct PixelCachePosition {
    pub x: usize,
    pub y: usize,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyNearPixel {
    pub distance: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenerationStatus {
    NotStarted,
    CopyNearPixel(CopyNearPixel),
    Unstable,
    Final,
}

#[derive(Clone, Debug)]
pub struct PixelCache {
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
    pub full_render_count: u64,
    pub index: usize,
    pub pixel_cache: Vec<PixelCache>,
    pub pixels_order: Vec<PixelCachePosition>,
    pub width: usize,
    pub height: usize,
}

pub struct RandomGenerator {
    data: GeneratorData,
}

impl RandomGenerator {
    pub fn new<R>(width: usize, height: usize, random: &mut R) -> RandomGenerator
    where
        R: rand::Rng + 'static + Send,
    {
        RandomGenerator {
            data: GeneratorData {
                full_render_count: 0,
                index: 0,
                pixel_cache: vec![
                    PixelCache {
                        last_color: None,
                        same_color_count: 0,
                        status: GenerationStatus::NotStarted,
                        nb_samples: 0,
                        incremental_raw_light: None,
                    };
                    (width * height) as usize
                ],
                pixels_order: get_random_positions(width as usize, height as usize, random),
                width,
                height,
            },
        }
    }
    pub fn invalidate_pixels<R>(&mut self, width: usize, height: usize, random: &mut R)
    where
        R: rand::Rng + 'static + Send,
    {
        let random_positions = vec![
            PixelCache {
                last_color: None,
                same_color_count: 0,
                status: GenerationStatus::NotStarted,
                nb_samples: 0,
                incremental_raw_light: None,
            };
            (width * height) as usize
        ];
        self.data.index = 0;
        self.data.pixel_cache = random_positions;
        self.data.full_render_count = 0;
        self.data.pixels_order = get_random_positions(width, height, random);
    }
    pub fn set_pixels_order(&mut self, width: usize, height: usize, positions: Vec<PixelCachePosition>) {
        self.data.index = 0;
        self.data.full_render_count = 0;
        self.data.pixels_order = positions;
    }
    pub fn propagate_pixels<S: PixelRenderer>(&mut self, renderer: &mut S) {
        // Propagate current pixel.
        for index in 0..self.data.pixel_cache.len() {
            let position_x = index % self.data.width;
            let position_y = index / self.data.width;
            //let pixel = &mut self.data.pixel_cache[index];
            if matches!(
                self.data.pixel_cache[index].status,
                GenerationStatus::CopyNearPixel(_) | GenerationStatus::NotStarted
            ) {
                continue;
            }
            if let Some(color) = self.data.pixel_cache[index].last_color.clone() {
                let propagate = 3;
                for x in (position_x - propagate)..(position_x + propagate) {
                    for y in (position_y - propagate)..(position_y + propagate) {
                        if let Some(near_index) = get_index(x,y, self.data.width, self.data.height) {
                            let near_pixel_cache = &mut self.data.pixel_cache[near_index];
                            if index == near_index {
                                continue;
                            }
                            let distance = ((position_x as i64 - (x as i64).abs()).abs() + (position_y as i64 - (y as i64).abs()).abs()) as usize;
                            if near_pixel_cache.status == GenerationStatus::NotStarted
                            {
                                near_pixel_cache.status = GenerationStatus::CopyNearPixel(CopyNearPixel{distance});
                                renderer.set_pixel(PixelPosition { x, y }, color.clone());
                            }
                            else if let GenerationStatus::CopyNearPixel(copy) = &mut near_pixel_cache.status {
                                if copy.distance <= distance {
                                    continue;
                                }
                                near_pixel_cache.status = GenerationStatus::CopyNearPixel(CopyNearPixel{distance});
                                renderer.set_pixel(PixelPosition { x, y }, color.clone());
                            }
                            // *///I am guessing These 2 lines would actually make the compiler optimize the ifs out.
                            //near_pixel_cache.status = GenerationStatus::CopyNearPixel(CopyNearPixel{distance});
                            //renderer.set_pixel(PixelPosition { x, y }, color.clone());
                        }
                    }
                }
            }
        }
    }
}

impl GeneratorProgress for RandomGenerator {
    fn get_pixel(&mut self) -> Option<(PixelCachePosition, &mut PixelCache)> {
        if self.data.pixels_order.len() == 0 {
            return None;
        }
        let position = self.data.pixels_order[self.data.index];
        Some((position, &mut self.data.pixel_cache[position.index]))
    }
    fn next(&mut self) -> Option<()> {
        self.data.index += 1;
        if self.data.index >= self.data.pixels_order.len() {
            self.data.index = 0;
            self.data.full_render_count += 1;
            None
        } else {
            Some(())
        }
    }
    fn get_index(&self) -> (u64, usize) {
        (self.data.full_render_count, self.data.index)
    }
}

pub trait GeneratorProgress {
    fn get_pixel(&mut self) -> Option<(PixelCachePosition, &mut PixelCache)>;
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
    ) -> Option<()> {
        let (pos, mut pixel) = generator.get_pixel()?;
        if pixel.status == GenerationStatus::Final {
            return Some(());
        }
        let mut samples_color = Vector3::new(0.0, 0.0, 0.0);
        for _s in 0..samples {
            let offset_x =
                (pos.x as f64 + self.info.random.gen_range(0.0, 1.0)) / (self.info.width - 1.0);
            let offset_y =
                (pos.y as f64 + self.info.random.gen_range(0.0, 1.0)) / (self.info.height - 1.0);
            let r = self.camera.emit_ray_at(offset_x, offset_y);
            samples_color += r.project_ray(&scene);
        }
        if let Some(incremental_raw_light) = pixel.incremental_raw_light {
            samples_color += incremental_raw_light;
        }
        pixel.incremental_raw_light = Some(samples_color);
        pixel.nb_samples += samples;
        let scale = 1.0 / (pixel.nb_samples) as f64;
        let corrected_pixel_color = (samples_color * scale)
            .map(|c| c.clamp(0.0, 1.0))
            .map(f64::sqrt)
            .map(|c| c * 255.0);
        let mut color = PixelColor::from(corrected_pixel_color);
        if let Some(last_color) = pixel.last_color.clone() {
            if last_color == color {
                pixel.same_color_count += 1;
            } else {
                pixel.same_color_count = 0;
            }
        } else {
            pixel.status = GenerationStatus::Unstable;
        }
        if pixel.same_color_count > MAX_SIMILAR_SAMPLE_FOR_A_PIXEL {
            pixel.status = GenerationStatus::Final;
        }
        color.status = pixel.status.clone();
        pixel.last_color = Some(color.clone());
        renderer.set_pixel(PixelPosition { x: pos.x, y: pos.y }, color);
        Some(())
    }
}

pub fn get_index(x: usize, y: usize, width: usize, height: usize) -> Option<usize> {
    if x >= width || y >= height {
        None
    }
    else {
        Some(x + y * width)
    }
}

fn get_random_positions<R>(width: usize, height: usize, rng: &mut R) -> Vec<PixelCachePosition>
where
    R: rand::Rng + 'static + Send,
{
    let mut positions: Vec<PixelCachePosition> = Vec::with_capacity(height * width);
    let mut index = 0;
    for y in 0..height {
        for x in 0..width {
            positions.push(PixelCachePosition { x, y, index });
            index += 1;
        }
    }
    positions.shuffle(rng);
    positions
}
pub fn get_positions_around<R>(width: usize, height: usize, rng: &mut R, x: usize, y: usize, radius: usize) -> Vec<PixelCachePosition>
where
    R: rand::Rng + 'static + Send,
{
    let mut positions: Vec<PixelCachePosition> = Vec::with_capacity(radius * 4);
    let radius = radius as i64;
    let origin_x = x as i64;
    let origin_y = y as i64;
    let sq_radius = radius * radius;
    for y_offset in -radius..radius {
        let distance_y = y_offset * y_offset;
        for x_offset in -radius..radius {
            let sq_distance = distance_y + x_offset * x_offset;
            if sq_distance >= sq_radius {
                continue;
            }
            let x = origin_x + x_offset;
            let y = origin_y + y_offset;
            if x < 0 || y < 0 {
                continue;
            }
            if let Some(index) = get_index(x as usize, y as usize, width, height) {
                positions.push(PixelCachePosition { x: x as usize, y: y as usize, index });
            }
        }
    }
    positions.shuffle(rng);
    positions
}
