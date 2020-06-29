use std::f64::consts::PI;

use nalgebra::Vector3;

use crate::materials::material::Material;
use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};

pub struct Metal {
    albedo: f64, // in fact this is not really a Color, more a RGB % of reflection
}

impl Metal {
    pub fn new() -> Metal {
        Metal { albedo: 0.8 / PI }
    }

    fn reflect(&self, ray: &Vector3<f64>, normal: &Vector3<f64>) -> Vector3<f64> {
        ray - 2.0 * ray.dot(normal) * normal
    }
}

impl Material for Metal {
    fn scatter(&self, _ray: &Ray, collision: &Collision) -> Color {
        // let target = collision.normal() + random_unit_vector();
        let _light_vector = collision.normal(); // global lightning, could consider normal to be // with light
        let light_intensity = 3.0; // global lightning, to be changed
        let light_color = Color::new(1.0, 1.0, 1.0); // global lightning, to be changed

        light_color * self.albedo * light_intensity
    }

    fn bounce(&self, ray: &Ray, collision: &Collision) -> Ray {
        Ray::new(
            *collision.position(),
            self.reflect(&ray.direction().normalize(), &collision.normal()),
        )
    }
}
