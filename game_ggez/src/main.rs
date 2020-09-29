pub mod poisson;

use poisson::Poisson;

use event::{KeyCode, KeyMods, MouseButton};
use ggez::event::{self, EventHandler};
use ggez::input::keyboard;
use ggez::{graphics, nalgebra::Point2, Context, ContextBuilder, GameResult};
use graphics::Text;
use rand::{Rng, SeedableRng, prelude::SmallRng};
use raytracer_core::{
    materials::{dielectric::Dielectric, lambertian_diffuse::Lambertian, metal::Metal},
    shapes::sphere::Sphere,
    Shape,
    GeneratorProgress, PixelRenderer, RandomGenerator, Raytracer, Scene, Vector3,
};

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
        let pixels = vec![0; PIXELS_ARRAY_SIZE];
        Self { pixels }
    }
}
impl raytracer_core::PixelRenderer for Renderer {
    fn set_pixel(
        &mut self,
        pos: raytracer_core::PixelPosition,
        new_pixel: raytracer_core::PixelColor,
    ) {
        if pos.x >= WIDTH || pos.y >= HEIGHT {
            return;
        }
        // NOTE: this is not thread safe
        let index = (PIXELS_ARRAY_SIZE - 4) - (((WIDTH - pos.x - 1) * 4) + pos.y * WIDTH * 4);
        self.pixels[index] = new_pixel.r;
        self.pixels[index + 1] = new_pixel.g;
        self.pixels[index + 2] = new_pixel.b;
        self.pixels[index + 3] = 255;
    }

    fn invalidate_pixels(&mut self) {
        self.pixels = vec![0; PIXELS_ARRAY_SIZE];
    }
}
struct MyGame {
    renderer: Renderer,
    raytracer: Raytracer<SmallRng>,
    generator: RandomGenerator,
    shapes: Vec<Box<dyn raytracer_core::Shape>>,
    scene: Scene,
    time_next_frame: std::time::Duration,
    random: SmallRng,
    current_eye_radius: f64,
    target_eye_radius: f64,
    target_shape_index: usize,
    must_invalidate: bool,
}

impl MyGame {
    pub fn new(
        _ctx: &mut Context,
        dimensions: Dimensions,
        target_shape_index: usize,
    ) -> MyGame {
        let rng = SmallRng::from_entropy();
        let raytracer = Raytracer::new(dimensions.width as f64, dimensions.height as f64, rng);

        let generator = RandomGenerator::new(
            dimensions.width,
            dimensions.height,
            &mut SmallRng::from_entropy(),
        );
        MyGame {
            renderer: Renderer::new(),
            raytracer,
            generator,
            shapes: vec![],
            scene: vec![],
            time_next_frame: ggez::timer::f64_to_duration(0_f64),
            random: SmallRng::from_entropy(),
            current_eye_radius: 0_f64,
            target_eye_radius: 100_f64,
            target_shape_index,
            must_invalidate: false,
        }
    }
    fn change_scene(&mut self, seed: u64) {
        let ground = Sphere::new(
            Vector3::new(0.0, -100.5, -1.0),
            100.0,
            Box::new(Lambertian::new_from_hex(0x007070)),
        );

        let mut shapes: Scene = vec![Box::new(ground)];
        let mut positions = vec![(0f64, 5f64), (0f64, 0f64)];
        let poisson = Poisson::new();
        let mut index = 0;
        let nb_new_shapes = 40;
        self.target_shape_index = self.random.gen_range(1, nb_new_shapes - 2);
        while index < positions.len() && shapes.len() < nb_new_shapes {
            let ref_point = positions[index];
            if let Some(new_position) = poisson.compute_new_position(&positions, &ref_point, 3f64, 10, &mut self.random) {
                
                let new_shape = if shapes.len() == self.target_shape_index {
                    Sphere::new(
                        Vector3::new(new_position.0, 0.0, new_position.1),
                        0.5,
                        Box::new(Dielectric::new(Vector3::new(1.0, 0.6, 0.60), 1.05)),
                    )
                }
                else {
                    Sphere::new(
                        Vector3::new(new_position.0, 0.0, new_position.1),
                        0.5,
                        Box::new(Dielectric::new(Vector3::new(0.0, 0.6, 1.0), 1.5)),
                    )
                };
                shapes.push(Box::new(new_shape));
                positions.push(new_position);
            }
            else {
                index += 1;
            }
        }
        dbg!(shapes.len());
        self.scene = shapes;
    }
}

impl EventHandler for MyGame {
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        match _button {
            MouseButton::Left => {
                let mouse_position = ggez::input::mouse::position(_ctx);
                if let Some(shape_index) = self.raytracer.get_shape(
                    &self.scene,
                    mouse_position.x as f64,
                    HEIGHT as f64 - mouse_position.y as f64,
                ) {
                    if shape_index == self.target_shape_index {
                        let random_seed = self.random.gen();
                        self.change_scene(random_seed);
                        self.must_invalidate = true;
                        self.current_eye_radius = 0_f64;
                    } else {
                        self.target_eye_radius = self.current_eye_radius / 2_f64;
                    }
                }
                else {
                        self.target_eye_radius = self.current_eye_radius / 2_f64;
                }
            }
            _ => {}
        }
    }
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let eye_radius_moving = 75f64;
        let mut time_since_start = ggez::timer::time_since_start(_ctx);
        let time_begin_frame = time_since_start;
        // input code here...
        if keyboard::is_key_pressed(_ctx, KeyCode::Up) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 5_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .move_camera(Vector3::new(0_f64, 0_f64, movement))
            } else {
                let movement = 1_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .move_camera(Vector3::new(0_f64, 0_f64, movement))
            }
            self.target_eye_radius = eye_radius_moving;
            self.must_invalidate = true;
        } else if keyboard::is_key_pressed(_ctx, KeyCode::Down) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 3_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .move_camera(Vector3::new(0_f64, 0_f64, -movement))
            } else {
                let movement = 1_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .move_camera(Vector3::new(0_f64, 0_f64, -movement))
            }
            self.target_eye_radius = eye_radius_moving;
            self.must_invalidate = true;
        }
        if keyboard::is_key_pressed(_ctx, KeyCode::Left) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 1.5_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .rotate(Vector3::new(0_f64, movement, 0_f64))
            } else {
                let movement = 0.75_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .rotate(Vector3::new(0_f64, movement, 0_f64))
            }
            self.target_eye_radius = eye_radius_moving;
            self.must_invalidate = true;
        } else if keyboard::is_key_pressed(_ctx, KeyCode::Right) {
            if keyboard::is_mod_active(_ctx, KeyMods::SHIFT) {
                let movement = 1.5_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .rotate(Vector3::new(0_f64, -movement, 0_f64))
            } else {
                let movement = 0.75_f64 * ggez::timer::delta(_ctx).as_secs_f64();
                self.raytracer.camera = self
                    .raytracer
                    .camera
                    .rotate(Vector3::new(0_f64, -movement, 0_f64))
            }
            self.target_eye_radius = eye_radius_moving;
            self.must_invalidate = true;
        }
        self.current_eye_radius = 2000f64;
        // FIXME: #pixelcache: This condition should exist to avoid cleaning correct pixels
        if self.must_invalidate
        {
            self.generator
                .invalidate_pixels(WIDTH, HEIGHT, &mut self.random);
            // NOTE: not invalidating renderer pixels is a great way to gain performance with very minimal visual impact
            // ----> Also, some might argue that the visual is better WITHOUT invalidating renderer pixels.
            // ----> without a smart #pixelcache solution though, we don't have much choice.
            //self.renderer.invalidate_pixels();
            // FIXME: #pixelcache: dirty hack to take radius into account
            if self.must_invalidate {
                self.must_invalidate = false;
            }
        }
        self.current_eye_radius = move_towards(
            self.current_eye_radius,
            self.target_eye_radius,
            300_f64 * ggez::timer::delta(_ctx).as_secs_f64(),
        );
        self.target_eye_radius = move_towards(
            self.target_eye_radius,
            200_f64,
            150_f64 * ggez::timer::delta(_ctx).as_secs_f64(),
        );

        let mouse_position = ggez::input::mouse::position(_ctx);
        let radius = self.current_eye_radius as usize;
        /*let positions_around_mouse = raytracer_core::get_positions_around(
            WIDTH,
            HEIGHT,
            &mut self.random,
            mouse_position.x as usize,
            HEIGHT - mouse_position.y as usize,
            radius,
        );*/
        //self.generator
        //    .set_pixels_order(WIDTH, HEIGHT, positions_around_mouse);

        // Update code here...
        let mut retries = 0;
        const PIXELS: u32 = 1300;
        let can_propagate = self.generator.get_index().0 == 0;
        while time_since_start < self.time_next_frame {
            let mut i = 0;
            while i < PIXELS {
                self.raytracer.generate_pixel(
                    &mut self.generator,
                    &self.scene,
                    1,
                    &mut self.renderer,
                );
                self.generator.next();
                i += 1;
            }
            retries += 1;
            time_since_start = ggez::timer::time_since_start(_ctx)
        }
        if can_propagate {
            self.generator.propagate_pixels(&mut self.renderer);
        }
        time_since_start = ggez::timer::time_since_start(_ctx);
        let time_for_frame = time_since_start - time_begin_frame;
        //println!("pixels: {} ; {} retries ; {} fps", retries * pixels, retries, ggez::timer::fps(_ctx));
        // FIXME: this fps calculation doesn't take into account time to (render + other work) (so the fps can drop significantly)
        // The fix would be to estimate the other work and substract it to time_next_frame.
        // Also, if the raytracer is done for current image, we should sleep!
        let target_fps = 20_f64;
        self.time_next_frame = time_since_start + ggez::timer::f64_to_duration(1_f64 / target_fps)
            - (time_for_frame / 10);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        /*let pixels: Vec<[u8;4]> = self.generator.get_raw_pixels().iter().map(|p| {
            if let Some(color) = &p.last_color {
                [color.r, color.g, color.b, 255]
            }
            else {
                [0,0,0,0]
            }
        }).collect();
        let pixels: Vec<u8> = pixels.iter().flatten().copied().collect();*/
        let pixels = &self.renderer.pixels;
        let image = ggez::graphics::Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, pixels)?;
        graphics::draw(ctx, &image, ggez::graphics::DrawParam::new())?;
        let fps = Text::new(format!("{:.1}", ggez::timer::fps(ctx)));
        graphics::draw(
            ctx,
            &fps,
            ggez::graphics::DrawParam::new().color(graphics::Color::from_rgb(255, 0, 0)),
        )?;
        let bg = ggez::graphics::Image::solid(ctx, 50, ggez::graphics::BLACK)?;
        graphics::draw(
            ctx,
            &bg,
            ggez::graphics::DrawParam::new()
                .dest(Point2::new(0.0 - 2.0, 37.0 - 2.0))
                .scale([1.2, 1.2])
                .color(graphics::Color::from_rgb(0, 0, 0)),
        )?;
        let progress = self.generator.get_index();
        let ratio = Text::new(format!(
            "{}, {:.1}",
            progress.0,
            progress.1 as f64 / (WIDTH * HEIGHT) as f64 * 100_f64
        ));
        graphics::draw(
            ctx,
            &ratio,
            ggez::graphics::DrawParam::new()
                .dest(Point2::new(0.0, 37.0))
                .color(graphics::Color::from_rgb(255, 255, 255)),
        )?;
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
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(
        &mut ctx,
        Dimensions {
            height: HEIGHT,
            width: WIDTH,
        },
        0,
    );
    my_game.change_scene(0);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

fn move_towards(orig: f64, target: f64, amount: f64) -> f64 {
    if orig < target {
        f64::min(orig + amount, target)
    } else if orig > target {
        f64::max(orig - amount, target)
    } else {
        target
    }
}