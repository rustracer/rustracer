use crate::collision::Collision;
use crate::materials::material::Material;
use crate::ray::Ray;
use nalgebra::Vector3;

pub trait Shape {
    fn collide(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Collision>;

    fn normal_at_position(&self, position: &Vector3<f64>) -> Vector3<f64>;

    fn material(&self) -> &dyn Material;
}
