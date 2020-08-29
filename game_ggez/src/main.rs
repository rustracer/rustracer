use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use raytracer_core::{shapes::sphere::Sphere, Vector3, materials::{lambertian_diffuse::Lambertian, dielectric::Dielectric, metal::Metal}, Scene, Raytracer, Generator};
use rand::{SeedableRng, prelude::SmallRng};

const WIDTH: usize = 1920 / 2;
const HEIGHT: usize = 1080 / 2;
const PIXELS_ARRAY_SIZE: usize = WIDTH * HEIGHT * 4;

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
pub struct Renderer {
    pixels: [u8; PIXELS_ARRAY_SIZE],
}

impl Renderer {
    fn new() -> Self {
        let mut pixels = [0;PIXELS_ARRAY_SIZE];
        Self {
            pixels,
        }
    }
}
impl raytracer_core::PixelRenderer for Renderer {
    fn set_pixel(&mut self, pos: raytracer_core::PixelPosition, new_pixel: raytracer_core::PixelColor) {
        // NOTE: this is not thread safe
        let index = pos.x * 4 + pos.y * WIDTH * 4;
        //println!("index: {}", index);
        self.pixels[index] = new_pixel.r;
        self.pixels[index + 1] = new_pixel.g;
        self.pixels[index + 2] = new_pixel.b;
        self.pixels[index + 3] = 255;
    }

    fn invalidate_pixels(&mut self) {
        self.pixels = [255;PIXELS_ARRAY_SIZE];
    }
}
struct MyGame<'a> {
    renderer: Renderer,
    raytracer: Raytracer<SmallRng>,
    generator: Generator,
    scene: Scene<'a>,
}

impl<'a> MyGame<'a> {
    pub fn new(_ctx: &mut Context, dimensions: Dimensions, scene: Scene<'a>) -> MyGame<'a> {
        let rng = SmallRng::from_entropy();
        let raytracer = Raytracer::new(dimensions.width as f64, dimensions.height as f64, rng);

        let mut generator = raytracer.get_new_generator();
        MyGame {
            renderer: Renderer::new(),
            raytracer,
            generator,
            scene,
        }
    }
}

impl<'a> EventHandler for MyGame<'a> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        while(ggez::timer::check_update_time(_ctx, (WIDTH * HEIGHT) as u32)) {
            if let Some(pixel_result) = self.raytracer.generate_pixel(&mut self.generator, &self.scene, 1, &mut self.renderer)
            {
                self.generator.index = self.generator.index + 1;
            } else {
                self.generator.index = 0;
            }
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let image = ggez::graphics::Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, &self.renderer.pixels).unwrap();
        graphics::draw(ctx, &image, ggez::graphics::DrawParam::new());
        // Draw code here...
        graphics::present(ctx)
    }
}

fn main() {
    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("Rustracer game", "Cool Game Author")
        .window_setup(ggez::conf::WindowSetup {
            title: "An easy, good game".to_owned(),
            samples: ggez::conf::NumSamples::Zero,
            vsync: false,
            icon: "".to_owned(),
            srgb: true,
        })
        .build()
		.expect("aieee, could not create ggez context!");

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
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(&mut ctx, Dimensions{height: HEIGHT, width: WIDTH }, scene);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}