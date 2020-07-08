use std::f64::consts::{PI, TAU};

use nalgebra::Vector3;

use crate::materials::material::Material;
use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;

pub struct Metal {
    albedo: Vector3<f64>, // in fact this is not really a Color, more a RGB % of reflection
    fuzziness: f64,
    rng: RefCell<SmallRng>,
}

impl Metal {
    pub fn new(albedo: Vector3<f64>, fuzziness: f64) -> Metal {
        Metal {
            albedo: albedo / PI,
            fuzziness,
            rng: RefCell::new(SmallRng::from_entropy()),
        }
    }

    fn reflect(&self, ray: &Vector3<f64>, normal: &Vector3<f64>) -> Vector3<f64> {
        ray - 2.0 * ray.dot(normal) * normal
    }

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

impl Material for Metal {
    fn scatter(&self, _ray: &Ray, collision: &Collision) -> Color {
        // let target = collision.normal() + random_unit_vector();
        let _light_vector = collision.normal(); // global lightning, could consider normal to be // with light
        let light_intensity = 3.0; // global lightning, to be changed
        let light_color = Color::new(1.0, 1.0, 1.0); // global lightning, to be changed

        Vector3::new(
            self.albedo.x * light_color.x,
            self.albedo.y * light_color.y,
            self.albedo.z * light_color.z,
        ) * light_intensity
    }

    fn bounce(&self, ray: &Ray, collision: &Collision) -> Option<Ray> {
        let reflected = self.reflect(&ray.direction().normalize(), &collision.normal());
        let direction: Vector3<f64> = reflected + self.fuzziness * self.random_unit_vector();

        if direction.dot(&collision.normal()) < 0.0 {
            return None;
        }
        Some(Ray::new(*collision.position(), direction))
    }
}
