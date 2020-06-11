#![feature(tau_constant)]
#![feature(clamp)]

use std::thread;

use raytracer_core::Vector3;

use crate::renderers::pixels::RendererPixels;
use crate::renderers::renderer::{Dimensions, Renderer};
use raytracer_core::shapes::sphere::Sphere;
use raytracer_core::{Scene, Raytracer};

use lazy_static::lazy_static;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::sync::Mutex;

mod renderers;

const SAMPLES_PER_PIXEL: i64 = 64;

lazy_static! {
    pub static ref RNG: Mutex<SmallRng> = Mutex::new(SmallRng::from_entropy());
}

pub fn rand_range_f64(start: f64, stop: f64) -> f64 {
    RNG.lock().unwrap().gen_range(start, stop)
}
    
fn main_loop() {
    let width = 1920.0 / 2.0;
    let height = 1080.0 / 2.0;

    let mut renderer = RendererPixels::new(Dimensions {
        height: height as usize,
        width: width as usize,
    });

    let mut set_pixel = renderer.pixel_accessor();
    eprint!("Scanlines remaining:\n");
    let handle = thread::spawn(move || {
        let sphere = Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5);
        let sphere2 = Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0);
        let sphere3 = Sphere::new(Vector3::new(0.5, -0.4, -0.85), 0.1);
        let scene: Scene = vec![&sphere, &sphere2, &sphere3];
        let raytracer = Raytracer{};
        raytracer.generate(width, height, scene, SAMPLES_PER_PIXEL, set_pixel, rand_range_f64);
    });
    renderer.start_rendering();

    eprint!("\nDone! :-)\n");
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
