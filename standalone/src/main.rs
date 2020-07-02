#![feature(tau_constant)]
#![feature(clamp)]

use std::thread;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use raytracer_core::materials::dielectric::Dielectric;
use raytracer_core::materials::lambertian_diffuse::Lambertian;
use raytracer_core::materials::metal::Metal;
use raytracer_core::shapes::sphere::Sphere;
use raytracer_core::Vector3;
use raytracer_core::{Raytracer, Scene};

use crate::renderers::pixels::RendererPixels;
use crate::renderers::renderer::{Dimensions, Renderer};

mod renderers;

const SAMPLES_PER_PIXEL: i64 = 300;

fn main_loop() {
    let width = 1920.0 / 2.0;
    let height = 1080.0 / 2.0;

    let mut renderer = RendererPixels::new(Dimensions {
        height: height as usize,
        width: width as usize,
    });
    let set_pixel = renderer.pixel_accessor();
    eprint!("Scanlines remaining:\n");
    thread::spawn(move || {
        let sphere = Sphere::new(
            Vector3::new(-1.01, 0.0, -1.0),
            0.5,
            Box::new(Dielectric::new(Vector3::new(1.0, 0.90, 0.90), 1.02)),
        );
        let sphere2 = Sphere::new(
            Vector3::new(0.0, -100.5, -1.0),
            100.0,
            Box::new(Lambertian::new_from_hex(0x007070)),
        );
        let sphere3 = Sphere::new(
            Vector3::new(1.0, 0.0, -1.0),
            0.5,
            Box::new(Metal::new(Vector3::new(0.8, 0.8, 0.8), 0.1)),
        );
        let sphere4 = Sphere::new(
            Vector3::new(-0.0, 0.0, -1.0),
            0.5,
            Box::new(Metal::new(Vector3::new(0.8, 0.6, 0.2), 0.5)),
        );

        let scene: Scene = vec![&sphere, &sphere2, &sphere3, &sphere4];
        let rng = &mut SmallRng::from_entropy();

        let raytracer = Raytracer::new(width, height, rng);

        for _depth in 0..=SAMPLES_PER_PIXEL {
            raytracer.generate(scene.as_slice(), 100, &set_pixel, rng);
        }
        eprintln!("OK");
    });
    renderer.start_rendering();
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
