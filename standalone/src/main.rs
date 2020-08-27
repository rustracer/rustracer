#![feature(tau_constant)]
#![feature(clamp)]

use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use raytracer_core::materials::dielectric::Dielectric;
use raytracer_core::materials::lambertian_diffuse::Lambertian;
use raytracer_core::materials::metal::Metal;
use raytracer_core::shapes::sphere::Sphere;
use raytracer_core::Vector3;
use raytracer_core::{PixelColor, PixelPosition, Raytracer, Scene};
use renderers::pixels::World;

use crate::renderers::pixels::RendererPixels;
use crate::renderers::renderer::{Command, Dimensions, Renderer};

mod renderers;

// const SAMPLES_PER_PIXEL: i64 = 300;

pub struct PixelRendererCommunicator {
    world: Arc<RwLock<World>>,
}

impl PixelRendererCommunicator {
    fn new(world: Arc<RwLock<World>>) -> Self {
        Self { world }
    }
}

impl raytracer_core::PixelRenderer for PixelRendererCommunicator {
    fn set_pixel(&mut self, pos: PixelPosition, color: PixelColor) {
        let mut world = self.world.write().unwrap();
        world.set_pixel(pos.x, pos.y, color)
    }
    fn invalidate_pixels(&mut self) {
        let mut world = self.world.write().unwrap();
        world.invalidate_pixels();
    }
}

fn main_loop() {
    let width = 1920.0 / 2.0;
    let height = 1080.0 / 2.0;

    let (tx, rx) = mpsc::channel();
    let mut renderer = RendererPixels::new(
        Dimensions {
            height: height as usize,
            width: width as usize,
        },
        tx,
    );
    eprint!("Scanlines remaining:\n");
    let communicator = renderer.pixel_accessor();

    thread::spawn(move || {
        let sphere = Sphere::new(
            Vector3::new(-1.01, 0.0, -1.0),
            0.5,
            Box::new(Dielectric::new(Vector3::new(1.0, 0.6, 0.60), 1.05)),
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
        let mut spp = 1;
        let rng = SmallRng::from_entropy();
        let mut raytracer = Raytracer::new(width, height, rng, communicator);

        let mut generator = raytracer.get_new_generator();
        loop {
            //spp *= 2;
            if let Some(pixel_result) =
                raytracer.generate_pixel(&mut generator, scene.as_slice(), spp)
            {
                generator.index = generator.index + 1;
            } else {
                generator.index = 0;
            }
            while let Ok(received_command) = rx.try_recv() {
                spp = 1;
                raytracer.invalidate_pixels();
                // frame dependant is bad but it does the job.
                raytracer.camera = match received_command {
                    Command::Move(movement) => raytracer
                        .camera
                        .move_camera(Vector3::new(movement.x, movement.y, movement.z)),
                    Command::Rotate(rotation) => raytracer
                        .camera
                        .rotate(Vector3::new(rotation.x, rotation.y, rotation.z)),
                }
            }
        }
    });
    renderer.start_rendering();
}

fn main() -> std::io::Result<()> {
    main_loop();
    Ok(())
}
