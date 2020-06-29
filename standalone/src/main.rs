#![feature(tau_constant)]
#![feature(clamp)]

use std::thread;

use raytracer_core::Vector3;

use crate::renderers::pixels::RendererPixels;
use crate::renderers::renderer::{Dimensions, Renderer};
use raytracer_core::shapes::sphere::Sphere;
use raytracer_core::{Raytracer, Scene};

use rand::rngs::SmallRng;
use rand::SeedableRng;

mod renderers;

const SAMPLES_PER_PIXEL: i64 = 6;

fn main_loop() {
    let width = 1920.0 / 2.0;
    let height = 1080.0 / 2.0;

    let mut renderer = RendererPixels::new(Dimensions {
        height: height as usize,
        width: width as usize,
    });

    let set_pixel_1 = renderer.pixel_accessor(1.0);
    let set_pixel_2 = renderer.pixel_accessor(1.0 - 1.0 / (SAMPLES_PER_PIXEL as f32 + 1.0));
    let set_pixel_3 = renderer.pixel_accessor(1.0 - (SAMPLES_PER_PIXEL as f32 + 1.0) / ((SAMPLES_PER_PIXEL * 2) as f32 + 1.0));
    let set_pixel_4 = renderer.pixel_accessor(1.0 - (SAMPLES_PER_PIXEL as f32 * 2.0 + 1.0) / ((SAMPLES_PER_PIXEL * 3) as f32 + 1.0));
    let set_pixel_5 = renderer.pixel_accessor(1.0 - (SAMPLES_PER_PIXEL as f32 * 3.0 + 1.0) / ((SAMPLES_PER_PIXEL * 4) as f32 + 1.0));
    let set_pixel_6 = renderer.pixel_accessor(1.0 - (SAMPLES_PER_PIXEL as f32 * 4.0 + 1.0) / ((SAMPLES_PER_PIXEL * 5) as f32 + 1.0));
    let set_pixel_7 = renderer.pixel_accessor(1.0 - (SAMPLES_PER_PIXEL as f32 * 5.0 + 1.0) / ((SAMPLES_PER_PIXEL * 6) as f32 + 1.0));
    let set_pixel_8 = renderer.pixel_accessor(1.0 - (SAMPLES_PER_PIXEL as f32 * 6.0 + 1.0) / ((SAMPLES_PER_PIXEL * 7) as f32 + 1.0));
    eprint!("Scanlines remaining:\n");
    thread::spawn(move || {
        let sphere = Sphere::new_with_metal(Vector3::new(0.0, 0.0, -1.0), 0.5);
        let sphere2 = Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0);
        let sphere3 = Sphere::new(Vector3::new(0.5, -0.4, -0.85), 0.1);
        let scene: Scene = vec![&sphere, &sphere2, &sphere3];
        let raytracer = Raytracer{};
        raytracer.generate(width, height, scene.clone(), 1, set_pixel_1, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene.clone(), SAMPLES_PER_PIXEL, set_pixel_2, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene.clone(), SAMPLES_PER_PIXEL, set_pixel_3, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene.clone(), SAMPLES_PER_PIXEL, set_pixel_4, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene.clone(), SAMPLES_PER_PIXEL, set_pixel_5, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene.clone(), SAMPLES_PER_PIXEL, set_pixel_6, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene.clone(), SAMPLES_PER_PIXEL, set_pixel_7, &mut SmallRng::from_entropy());
        raytracer.generate(width, height, scene, SAMPLES_PER_PIXEL, set_pixel_8, &mut SmallRng::from_entropy());
        eprintln!("OK");
    });
    renderer.start_rendering();
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
