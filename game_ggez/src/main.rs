use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use raytracer_core::{shapes::sphere::Sphere, Vector3, materials::{lambertian_diffuse::Lambertian, dielectric::Dielectric, metal::Metal}, Scene, Raytracer};
use rand::{SeedableRng, prelude::SmallRng};

#[derive(Debug, Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
#[derive(Clone)]
struct Pixel {
    color: PixelColor,
    write_count: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub(crate) width: usize,
    pub(crate) height: usize,
}
struct Size {
    width: usize,
    height: usize,
}
pub struct World {
    pixels: Vec<Pixel>,
    size: Size,
    max_write_count: u64,
}

impl World {
    fn new(height: usize, width: usize) -> Self {
        let count = width * height;
        let black = PixelColor {
            r: 0,
            g: 0,
            b: 0,
        };
        let mut pixels = vec![Pixel {
            color: black,
            write_count: 0,
        };count];
        Self {
            pixels,
            size: Size { width, height },
            max_write_count: 1,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, new_pixel: PixelColor) {
        // NOTE: this is not thread safe
        let mut pixel = &mut self.pixels[y * self.size.width + x];
        pixel.color = new_pixel;
        pixel.write_count += 1;
        if pixel.write_count > self.max_write_count {
            self.max_write_count = pixel.write_count;
        }
    }

    pub fn invalidate_pixels(&mut self) {
        let black = PixelColor {
            r: 0,
            g: 0,
            b: 0,
        };
        for pixel in &mut self.pixels {
            pixel.color = black;
            pixel.write_count = 0;
        }
        self.max_write_count = 0;
    }
}
struct MyGame {
    world: World,
}

impl MyGame {
    pub fn new(_ctx: &mut Context, dimensions: Dimensions) -> MyGame {
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
        let mut raytracer = Raytracer::new(dimensions.width as f64, dimensions.height as f64, rng);

        let mut generator = raytracer.get_new_generator();
        MyGame {
            world: World::new(dimensions.height, dimensions.width),
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let image_raw = [
            255, 255, 255, 255,
            200, 200, 200, 255,
            150, 150, 150, 255,
            100, 100, 100, 255,
        ];
        let image = ggez::graphics::Image::from_rgba8(ctx, 2, 2, &image_raw).unwrap();
        graphics::draw(ctx, &image, ggez::graphics::DrawParam::new());
        // Draw code here...
        graphics::present(ctx)
    }
}


fn main() {
    let width = (1920.0 / 2.0) as usize;
    let height = (1080.0 / 2.0) as usize;
    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("Rustracer game", "Cool Game Author")
		.build()
		.expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(&mut ctx, Dimensions{height, width });

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}