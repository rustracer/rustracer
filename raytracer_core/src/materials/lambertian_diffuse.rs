use std::cell::RefCell;
use std::f64::consts::{PI, TAU};

use nalgebra::Vector3;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::materials::material::Material;
use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};

pub struct Lambertian {
    albedo: f64,
    // in fact this is not really a Color, more a RGB % of reflection
    rng: RefCell<SmallRng>,
}

impl Lambertian {
    pub fn new() -> Lambertian {
        Lambertian {
            albedo: 0.18 / PI,
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
        light_color * dot_product * self.albedo * light_intensity
    }

    fn bounce(&self, _ray: &Ray, collision: &Collision) -> Ray {
        Ray::new(
            *collision.position(),
            collision.normal() + self.random_unit_vector(),
        )
    }
}
