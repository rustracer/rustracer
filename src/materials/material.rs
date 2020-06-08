use crate::shapes::collision::Collision;
use crate::shapes::ray::{Color, Ray};

pub trait Material {
    fn scatter(&self, ray: &Ray, collision: &Collision) -> Color;

    fn bounce(&self, ray: &Ray, collision: &Collision) -> Ray;
}
