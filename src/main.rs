#![feature(tau_constant)]
#![feature(clamp)]

use nalgebra::Vector3;

use crate::camera::Camera;
use crate::collision::Collision;
use crate::materials::lambertian_diffuse::random_unit_vector;
use crate::rand_range_f64::rand_range_f64;
use crate::ray::Ray;
use crate::shapes::shape::Shape;
use crate::shapes::sphere::Sphere;

mod camera;
mod collision;
mod materials;
mod rand_range_f64;
mod ray;
mod shapes;

type Color = Vector3<f64>;

const SAMPLES_PER_PIXEL: i64 = 50;

fn write_color(color: Color) {
    let scale = 1.0 / SAMPLES_PER_PIXEL as f64;
    let ir = (255.999 * (color[0] * scale).clamp(0.0, 1.0).sqrt()) as i64;
    let ig = (255.999 * (color[1] * scale).clamp(0.0, 1.0).sqrt()) as i64;
    let ib = (255.999 * (color[2] * scale).clamp(0.0, 1.0).sqrt()) as i64;

    println!("{} {} {}", ir, ig, ib);
}

fn main_loop() {
    let camera = Camera::new();
    let width = 1920.0;
    let height = 1080.0;

    let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5);
    let sphere2 = Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0);
    let sphere3 = Sphere::new(Vector3::new(0.5, -0.4, -0.85), 0.1);
    let scene: Vec<&dyn Shape> = vec![&sphere, &sphere2, &sphere3];
    println!("P3\n{} {} \n255", width, height);

    eprint!("Scanlines remaining:\n");
    for y in (0..(height as i64)).rev() {
        eprint!("\r{} <= {}", height, height as i64 - y);
        for x in 0..(width as i64) {
            let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
            for _s in 0..SAMPLES_PER_PIXEL {
                let offset_x = (x as f64 + rand_range_f64(0.0, 1.0)) / (width - 1.0);
                let offset_y = (y as f64 + rand_range_f64(0.0, 1.0)) / (height - 1.0);
                let r = camera.emit_ray_at(offset_x, offset_y);
                pixel_color += r.project_ray(&scene);
            }
            write_color(pixel_color);
        }
    }
    eprint!("\nDone! :-)\n");
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
