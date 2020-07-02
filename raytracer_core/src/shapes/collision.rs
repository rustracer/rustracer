use nalgebra::Vector3;

use crate::shapes::ray::{Color, Ray};
use crate::shapes::shape::Shape;

pub struct Collision<'a> {
    dist_from_origin: f64,
    position: Vector3<f64>,
    shape: &'a dyn Shape,
}

impl Collision<'_> {
    pub fn new(dist_from_origin: f64, position: Vector3<f64>, shape: &dyn Shape) -> Collision {
        Collision {
            position,
            shape,
            dist_from_origin,
        }
    }

    pub fn position(&self) -> &Vector3<f64> {
        &self.position
    }

    pub fn dist_from_origin(&self) -> f64 {
        self.dist_from_origin
    }

    pub fn normal(&self) -> Vector3<f64> {
        self.shape.normal_at_position(self.position())
    }

    pub fn color(&self, ray: &Ray) -> Color {
        self.shape.material().scatter(ray, self)
    }

    pub fn bounce(&self, ray: &Ray) -> Option<Ray> {
        self.shape.material().bounce(ray, self)
    }
}
