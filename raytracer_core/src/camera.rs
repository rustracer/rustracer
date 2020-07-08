use crate::shapes::ray::Ray;
use nalgebra::Vector3;

const ASPECT_RATIO: f64 = 16.0 / 9.0;

pub struct Camera {
    pub origin: Vector3<f64>,
    lower_left_corner: Vector3<f64>,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
}

impl Camera {
    pub fn new(x: f64, y: f64, z: f64) -> Camera {
        let origin = Vector3::new(x, y, z);
        let lookat = Vector3::new(0.0, 0.0, -1.0);
        let vup = Vector3::new(0.0, 1.0, 0.0);
        let vertical_field_of_view = 20_f64;

        let viewport_height: f64 = 2.0 * vertical_field_of_view.to_radians();
        let viewport_width: f64 = ASPECT_RATIO * viewport_height;

        let w = (origin - lookat).normalize();
        let u = vup.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - horizontal / 2.0 - vertical / 2.0 - w,
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
