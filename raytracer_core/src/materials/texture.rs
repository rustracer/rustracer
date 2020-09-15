use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageResult, Pixel};

use crate::materials::material::Material;
use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};

enum Pattern {
    Simple,
    ExactFit,
}

pub struct Texture {
    image: DynamicImage,
    pattern: Pattern,
    scale: f64,
}

impl Texture {
    pub fn load_from_file(path: &Path, scale: f64) -> ImageResult<Texture> {
        match image::open(path) {
            Err(e) => Err(e),
            Ok(image) => Ok(Texture {
                image,
                scale,
                pattern: Pattern::Simple,
            }),
        }
    }

    fn wrap(&self, val: f64, bound: u32) -> u32 {
        let signed_bound = bound as i32;
        let float_coord = val * self.scale * bound as f64;
        let wrapped_coord = (float_coord as i32) % signed_bound;
        if wrapped_coord < 0 {
            (wrapped_coord + signed_bound) as u32
        } else {
            wrapped_coord as u32
        }
    }
}

impl Material for Texture {
    fn scatter(&self, _ray: &Ray, collision: &Collision) -> Color {
        let text_coord_on_shape = collision.texture_coordinates();

        let tex_x = self.wrap(text_coord_on_shape.x, self.image.width());
        let tex_y = self.wrap(text_coord_on_shape.y, self.image.height());
        let rgb = self.image.get_pixel(tex_x, tex_y).to_rgb();
        // println!("{:?}", rgb);
        Color::new(
            rgb[0] as f64 / 255.0,
            rgb[1] as f64 / 255.0,
            rgb[2] as f64 / 255.0,
        )
    }

    fn bounce(&self, _ray: &Ray, _collision: &Collision) -> Option<Ray> {
        None
    }
}
