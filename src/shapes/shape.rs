use crate::ray::Ray;
use nalgebra::Vector3;

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

    pub fn dist_from_origin(&self) -> &f64 {
        &self.dist_from_origin
    }

    pub fn normal(&self) -> Vector3<f64> {
        self.shape.normal_at_collision(self)
    }
}

pub trait Shape {
    fn collide(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Collision>;

    fn normal_at_collision(&self, collision: &Collision) -> Vector3<f64>;
}
