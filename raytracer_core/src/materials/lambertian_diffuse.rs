use std::cell::RefCell;
use std::f64::consts::{PI, TAU};

use nalgebra::Vector3;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::materials::material::Material;
use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};

pub struct Lambertian {
    albedo: Vector3<f64>,
    // in fact this is not really a Color, more a RGB % of reflection
    rng: RefCell<SmallRng>,
}

impl Lambertian {
    pub fn new(albedo: Vector3<f64>) -> Lambertian {
        Lambertian {
            albedo: albedo / PI,
            rng: RefCell::new(SmallRng::from_entropy()),
        }
    }

    pub fn new_from_hex(color: i64) -> Lambertian {
        Lambertian {
            albedo: Vector3::new(
                (((color & 0xFF0000) >> 16) as f64) / 255.0,
                (((color & 0x00FF00) >> 8) as f64) / 255.0,
                ((color & 0x0000FF) as f64) / 255.0,
            ) / PI,
            rng: RefCell::new(SmallRng::from_entropy()),
        }
    }
}

impl Lambertian {
    fn random_unit_vector(&self) -> Vector3<f64> {
        let a = self.rand_range_f64(0.0, TAU);
        let z = self.rand_range_f64(-1.0, 1.0);
        let r = (1.0 - z * z).sqrt();

        Vector3::new(r * a.cos(), r * a.sin(), z)
    }

    fn rand_range_f64(&self, start: f64, stop: f64) -> f64 {
        self.rng.borrow_mut().gen_range(start, stop)
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, collision: &Collision) -> Color {
        // let target = collision.normal() + random_unit_vector();
        let light_vector = collision.normal(); // global lightning, could consider normal to be // with light
        let light_intensity = 3.0; // global lightning, to be changed
        let light_color = Color::new(1.0, 1.0, 1.0); // global lightning, to be changed
        let dot_product = f64::max(0.0, collision.normal().dot(&light_vector));

        Vector3::new(
            self.albedo.x * light_color.x,
            self.albedo.y * light_color.y,
            self.albedo.z * light_color.z,
        ) * light_intensity
            * dot_product
    }

    fn bounce(&self, _ray: &Ray, collision: &Collision) -> Option<Ray> {
        Some(Ray::new(
            *collision.position(),
            collision.normal() + self.random_unit_vector(),
        ))
    }
}
