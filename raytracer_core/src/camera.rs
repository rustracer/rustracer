use crate::shapes::ray::Ray;
use nalgebra::Vector3;

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const VIEWPORT_HEIGHT: f64 = 2.0;
const VIEWPORT_WIDTH: f64 = ASPECT_RATIO * VIEWPORT_HEIGHT;
const FOCAL_LENGTH: f64 = 2.0;

pub struct Camera {
    origin: Vector3<f64>,
    lower_left_corner: Vector3<f64>,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
}

impl Camera {
    pub fn new() -> Camera {
        let origin = Vector3::new(0.0, 0.0, 1.0);
        let horizontal = Vector3::new(VIEWPORT_WIDTH, 0.0, 0.0);
        let vertical = Vector3::new(0.0, VIEWPORT_HEIGHT, 0.0);

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin
                - horizontal / 2.0
                - vertical / 2.0
                - Vector3::new(0.0, 0.0, FOCAL_LENGTH),
        }
    }
    pub fn emit_ray_at(&self, offset_x: f64, offset_y: f64) -> Ray {
        Ray::new(
            self.origin.clone_owned(),
            &self.lower_left_corner - &self.origin
                + &self.horizontal * offset_x
                + &self.vertical * offset_y,
        )
    }
}
