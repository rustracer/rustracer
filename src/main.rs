#![feature(test)]
#![feature(tau_constant)]

extern crate lazy_static;

use std::f64::consts::TAU;
use std::sync::Mutex;

use lazy_static::lazy_static;
use nalgebra::Vector3;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::camera::Camera;
use crate::ray::Ray;
use crate::shapes::shape::{Collision, Shape};
use crate::shapes::sphere::Sphere;

mod camera;
mod ray;
mod shapes;

#[cfg(test)]
use flame;

type Color = Vector3<f64>;

const T_MIN: f64 = 0.001;
const T_MAX: f64 = 100_000.0;
const SAMPLES_PER_PIXEL: i64 = 4;

lazy_static! {
    static ref RNG: Mutex<SmallRng> = Mutex::new(SmallRng::from_entropy());
}

fn random(start: f64, stop: f64) -> f64 {
    RNG.lock().unwrap().gen_range(start, stop)
}

fn find_collision<'a, 'b>(ray: &'a Ray, scene: &[&'b (dyn Shape + 'b)]) -> Option<Collision<'b>> {
    let mut maybe_collision: Option<Collision> = None;
    for shape in scene {
        let maybe_new_collision = shape.collide(ray, T_MIN, T_MAX);

        maybe_collision = match maybe_collision {
            Some(collision) => match maybe_new_collision {
                Some(new_collision)
                    if new_collision.dist_from_origin() < collision.dist_from_origin() =>
                {
                    Some(new_collision)
                }
                _ => Some(collision),
            },
            _ => maybe_new_collision,
        }
    }
    maybe_collision
}

fn random_unit_vector() -> Vector3<f64> {
    let a = random(0.0, TAU);
    let z = random(-1.0, 1.0);
    let r = (1.0 - z * z).sqrt();

    Vector3::new(r * a.cos(), r * a.sin(), z)
}

fn _project_ray(ray: &Ray, scene: &[&dyn Shape], depth: i64) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }
    #[cfg(test)]
    let _fg = ::flame::start_guard(format!("project_ray_depth_{}", depth.to_string()));
    let may_collision = find_collision(ray, scene);

    match may_collision {
        Some(collision) => {
            let target = collision.position() + collision.normal() + random_unit_vector();
            let diffusion_ray = Ray::new(
                collision.position().clone_owned(),
                target - collision.position(),
            );
            0.5 * _project_ray(&diffusion_ray, scene, depth - 1)
        }
        None => background_color(ray),
    }
}

fn project_ray(ray: &Ray, scene: &[&dyn Shape]) -> Color {
    // parameterize max depth
    _project_ray(ray, scene, 10)
}

fn sphere_color(normal: Vector3<f64>) -> Color {
    (normal + Color::new(1.0, 1.0, 1.0)) * 0.5
}

fn background_color(ray: &Ray) -> Color {
    let t = 0.5 * (ray.direction().normalize().y + 1.0);

    Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
}

fn write_color(color: Color) {
    let ir = (255.999 * color[0] / SAMPLES_PER_PIXEL as f64) as i64;
    let ig = (255.999 * color[1] / SAMPLES_PER_PIXEL as f64) as i64;
    let ib = (255.999 * color[2] / SAMPLES_PER_PIXEL as f64) as i64;

    println!("{} {} {}", ir, ig, ib);
}

pub fn main_loop(height: f64, width: f64) {
    let camera = Camera::new();
    let mut small_rng = SmallRng::from_entropy();

    let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5);
    let sphere2 = Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0);
    let sphere3 = Sphere::new(Vector3::new(0.5, -0.4, -0.85), 0.1);
    let scene: Vec<&dyn Shape> = vec![&sphere, &sphere2, &sphere3];
    println!("P3\n{} {} \n255", width, height);

    eprint!("Scanlines remaining:\n");
    for y in (0..(height as i64)).rev() {
        #[cfg(test)]
        let _fg = ::flame::start_guard("scan line");
        eprint!("\r{} <= {}", height, height as i64 - y);
        for x in 0..(width as i64) {
            #[cfg(test)]
            let _fg = ::flame::start_guard("pixel");
            let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
            for s in 0..SAMPLES_PER_PIXEL {
                #[cfg(test)]
                let _fg = ::flame::start_guard(format!("sample_{}", s));
                let _fg2 = ::flame::start_guard("random offsets");
                let offset_x = (x as f64 + small_rng.gen_range(0.0, 1.0)) / (width - 1.0);
                let offset_y = (y as f64 + small_rng.gen_range(0.0, 1.0)) / (height - 1.0);
                drop(_fg2);
                let r = camera.emit_ray_at(offset_x, offset_y);
                pixel_color += project_ray(&r, &scene);
            }
            write_color(pixel_color);
        }
    }
    eprint!("\nDone! :-)\n");
}

fn main() -> std::io::Result<()> {
    let height = 2160.0 / 30.0;
    let width = 3840.0 / 30.0;
    main_loop(height, width);
    Ok(())
}
#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use std::fs::File;

    use test::Bencher;

    #[test]
    fn gen_flame() {
        let _fg = ::flame::start_guard("main");
        
        main_loop(2160.0 / 200.0, 3840.0 / 200.0);
        // in order to create the flamegraph you must call one of the
        // flame::dump_* functions.
        flame::dump_html(File::create("flamegraph.html").unwrap()).unwrap();
    }
    #[bench]
    fn bench_simple(b: &mut Bencher) {
        b.iter(|| {
            let height = 2160.0;
        let width = 3840.0;
        main_loop(height / 30.0, width / 30.0);
        })
    }
}