use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use raytracer_core::{shapes::sphere::Sphere, Vector3, materials::{lambertian_diffuse::Lambertian, dielectric::Dielectric, metal::Metal}, Scene, Raytracer, Generator, PixelRenderer};
use rand::{SeedableRng, prelude::SmallRng};
use ggez::input::keyboard;
use event::{KeyMods, KeyCode};
use graphics::Text;

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
    // using Vec instead of [] because I had a stack overflow when using too many pixels
    pixels: Vec<u8>,
}

impl Renderer {
    fn new() -> Self {
        let mut pixels = vec![0;PIXELS_ARRAY_SIZE];
        Self {
            pixels,
        }
    }
}
impl raytracer_core::PixelRenderer for Renderer {
    fn set_pixel(&mut self, pos: raytracer_core::PixelPosition, new_pixel: raytracer_core::PixelColor) {
        // NOTE: this is not thread safe
        let index = (PIXELS_ARRAY_SIZE - 4) - (((WIDTH - pos.x - 1) * 4) + pos.y * WIDTH * 4);
        self.pixels[index] = new_pixel.r;
        self.pixels[index + 1] = new_pixel.g;
        self.pixels[index + 2] = new_pixel.b;
        self.pixels[index + 3] = 255;
    }

    fn invalidate_pixels(&mut self) {
        self.pixels = vec![0;PIXELS_ARRAY_SIZE];
    }
}
struct MyGame<'a> {
    renderer: Renderer,
    raytracer: Raytracer<SmallRng>,
    generator: Generator,
    scene: Scene<'a>,
    time_next_frame: std::time::Duration,
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
            time_next_frame: ggez::timer::f64_to_duration(0_f64),
        }
    }
}

impl<'a> EventHandler for MyGame<'a> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // input code here...
        if keyboard::is_key_pressed(_ctx, KeyCode::Up) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 5_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.move_camera(Vector3::new(0_f64, 0_f64, movement))
            }
            else {
                let movement = 1_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.move_camera(Vector3::new(0_f64, 0_f64, movement))
            }
            self.raytracer.invalidate_pixels();
            self.renderer.invalidate_pixels();
        } else if keyboard::is_key_pressed(_ctx, KeyCode::Down) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 3_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.move_camera(Vector3::new(0_f64, 0_f64, -movement))
            }
            else {
                let movement = 1_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.move_camera(Vector3::new(0_f64, 0_f64, -movement))
            }
            self.raytracer.invalidate_pixels();
            self.renderer.invalidate_pixels();
        }
        if keyboard::is_key_pressed(_ctx, KeyCode::Left) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 2_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.rotate(Vector3::new(0_f64, movement, 0_f64))
            }
            else {
                let movement = 0.5_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.rotate(Vector3::new(0_f64, movement, 0_f64))
            }
            self.raytracer.invalidate_pixels();
            self.renderer.invalidate_pixels();
        }
        else if keyboard::is_key_pressed(_ctx, KeyCode::Right) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 2_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.rotate(Vector3::new(0_f64, -movement, 0_f64))
            }
            else {
                let movement = 0.5_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self.raytracer.camera.rotate(Vector3::new(0_f64, -movement, 0_f64))
            }
            self.raytracer.invalidate_pixels();
            self.renderer.invalidate_pixels();
        }
        // Update code here...
        let mut time_since_start = ggez::timer::time_since_start(_ctx);
        let mut retries = 0;
        const pixels: u32 = 10000;
        while time_since_start < self.time_next_frame {
            //println!("looping");
            let mut i = 0;
            while i < pixels {
                if let Some(pixel_result) = self.raytracer.generate_pixel(&mut self.generator, &self.scene, 1, &mut self.renderer)
                {
                    self.generator.index = self.generator.index + 1;
                } else {
                    //println!("full screen filled");
                    self.generator.index = 0;
                }
                i = i + 1;
            }
            retries = retries + 1;
            time_since_start = ggez::timer::time_since_start(_ctx)
        }
        //println!("pixels: {} ; {} retries ; {} fps", retries * pixels, retries, ggez::timer::fps(_ctx));
        // FIXME: this fps calculation doesn't take into account time to (render + other work) (so the fps can drop significantly)
        // The fix would be to estimate the other work and substract it to time_next_frame.
        // Also, if the raytracer is done for current image, we should sleep!
        self.time_next_frame = time_since_start + ggez::timer::f64_to_duration(1_f64 / 10_f64);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let image = ggez::graphics::Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, &self.renderer.pixels).unwrap();
        graphics::draw(ctx, &image, ggez::graphics::DrawParam::new());
        let text = Text::new(format!("{:.1}", ggez::timer::fps(ctx)));
        graphics::draw(ctx, &text, ggez::graphics::DrawParam::new().color(graphics::Color::from_rgb(255,0,0)));
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