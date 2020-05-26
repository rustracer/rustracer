use nalgebra::Vector3;

use crate::collision::Collision;
use crate::ray::Ray;
use crate::Color;

pub trait Material {
    fn scatter(&self, ray: &Ray, collision: &Collision) -> Color;

    fn bounce(&self, ray: &Ray, collision: &Collision) -> Ray;
}
