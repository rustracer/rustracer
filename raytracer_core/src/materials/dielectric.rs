use std::cell::RefCell;
use std::f64::consts::PI;

use nalgebra::Vector3;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::materials::material::Material;
use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};

pub struct Dielectric {
    albedo: Vector3<f64>,
    // in fact this is not really a Color, more a RGB % of reflection
    refraction_idx: f64,
    rng: RefCell<SmallRng>,
}

impl Dielectric {
    pub fn new(albedo: Vector3<f64>, refraction_idx: f64) -> Dielectric {
        Dielectric {
            albedo: albedo / PI,
            rng: RefCell::new(SmallRng::from_entropy()),
            refraction_idx,
        }
    }

    fn reflect(ray: &Vector3<f64>, normal: &Vector3<f64>) -> Vector3<f64> {
        ray - 2.0 * ray.dot(normal) * normal
    }

    fn refract(
        ray: &Vector3<f64>,
        normal: &Vector3<f64>,
        refraction_index: f64,
    ) -> Option<Vector3<f64>> {
        let unit_vector = ray.normalize();
        let dt = unit_vector.dot(normal);
        let discriminant = 1.0 - refraction_index * refraction_index * (1.0 - dt * dt);

        if discriminant < 0.0 {
            return None;
        }
        Some((unit_vector - normal * dt) * refraction_index - normal * discriminant.sqrt())
    }

    fn shlick(cosine: f64, refraction_index: f64) -> f64 {
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        let r1 = r0 * r0;

        r1 + (1.0 - r1) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, _ray: &Ray, collision: &Collision) -> Color {
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
        let reflected = Dielectric::reflect(&ray.direction().normalize(), &collision.normal());

        let outward_normal;
        let refaction_index;
        let cosine;
        let dot_product = ray.direction().dot(&collision.normal());
        if dot_product > 0.0 {
            outward_normal = -collision.normal();
            refaction_index = self.refraction_idx;
            cosine = self.refraction_idx * dot_product / ray.direction().len() as f64
        } else {
            outward_normal = collision.normal();
            refaction_index = 1.0 / self.refraction_idx;
            cosine = -dot_product / ray.direction().len() as f64
        }

        let refracted = Dielectric::refract(ray.direction(), &outward_normal, refaction_index);

        let reflect_prob;
        if refracted.is_some() {
            reflect_prob = Dielectric::shlick(cosine, self.refraction_idx);
        } else {
            reflect_prob = 1.0;
        }

        if reflect_prob > self.rng.borrow_mut().gen_range(0.0, 1.0) {
            Some(Ray::new(*collision.position(), reflected))
        } else {
            Some(Ray::new(*collision.position(), refracted.unwrap()))
        }

        /*let refract_prop = Dielectric::shlick(cosine, self.refraction_idx);
        if refract_prop > self.rng.borrow_mut().gen_range(0.0, 1.0) {
            return match Dielectric::refract(ray.direction(), &outward_normal, refaction_index) {
                Some(refracted) => Some(Ray::new(*collision.position(), refracted)),
                None => Some(Ray::new(*collision.position(), reflected)),
            };
        }
        Some(Ray::new(*collision.position(), reflected))*/
    }
}
