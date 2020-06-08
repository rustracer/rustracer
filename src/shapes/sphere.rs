use nalgebra::Vector3;

use crate::materials::lambertian_diffuse::Lambertian;
use crate::materials::material::Material;

use super::shape::Shape;
use crate::shapes::collision::Collision;
use crate::shapes::ray::Ray;

pub struct Sphere {
    center: Vector3<f64>,
    radius: f64,
    material: Lambertian,
}

impl Sphere {
    pub fn new(center: Vector3<f64>, radius: f64) -> Sphere {
        Sphere {
            center,
            radius,
            material: Lambertian::new(),
        }
    }
}

impl Shape for Sphere {
    fn collide(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Collision> {
        let oc = ray.origin() - self.center;
        let a = ray.direction().magnitude_squared();
        let half_b = oc.dot(&ray.direction());
        let c = oc.magnitude_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant > 0.0 {
            let root = discriminant.sqrt();
            let hit_distance_from_ray_origin = (-half_b - root) / a;

            if hit_distance_from_ray_origin < t_max && hit_distance_from_ray_origin > t_min {
                let collision_origin = ray.at(hit_distance_from_ray_origin);

                return Some(Collision::new(
                    hit_distance_from_ray_origin,
                    collision_origin,
                    self,
                ));
            }
            let hit_distance_from_ray_origin = (-half_b + root) / a;
            if hit_distance_from_ray_origin < t_max && hit_distance_from_ray_origin > t_min {
                let collision_origin = ray.at(hit_distance_from_ray_origin);

                return Some(Collision::new(
                    hit_distance_from_ray_origin,
                    collision_origin,
                    self,
                ));
            }
        }
        None
    }

    fn normal_at_position(&self, position: &Vector3<f64>) -> Vector3<f64> {
        (position - self.center) / self.radius
    }

    fn material(&self) -> &dyn Material {
        &self.material
    }
}
