use crate::shapes::collision::Collision;
use crate::shapes::shape::Shape;
use nalgebra::Vector3;

const T_MIN: f64 = 0.001;
const T_MAX: f64 = 100_000.0;

pub type Color = Vector3<f64>;

trait Blendable {
    fn blend(&self, other: &Color) -> Color;
}

impl Blendable for Color {
    fn blend(&self, other: &Color) -> Self {
        Color::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

pub struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    pub fn new(origin: Vector3<f64>, direction: Vector3<f64>) -> Ray {
        Ray { origin, direction }
    }

    pub fn at(&self, t: f64) -> Vector3<f64> {
        self.direction * t + self.origin
    }

    pub fn origin(&self) -> &Vector3<f64> {
        &self.origin
    }

    pub fn direction(&self) -> &Vector3<f64> {
        &self.direction
    }

    pub fn project_ray(&self, scene: &[&dyn Shape]) -> Color {
        // parameterize max depth
        self._project_ray(scene, 50)
    }

    pub(crate) fn find_collision<'a>(
        &self,
        scene: &[&'a (dyn Shape + 'a)],
    ) -> Option<(Collision<'a>, usize)> {
        let mut maybe_collision: Option<(Collision, usize)> = None;
        let mut index = 0;
        for shape in scene {
            let maybe_new_collision = shape.collide(self, T_MIN, T_MAX);

            if let Some(new_collision) = maybe_new_collision {
                match &maybe_collision {
                    Some(old_collision)
                        if old_collision.0.dist_from_origin()
                            < new_collision.dist_from_origin() => {}
                    _ => {
                        maybe_collision = Some((new_collision, index));
                    }
                }
            };
            index += 1;
        }
        maybe_collision
    }

    fn _project_ray(&self, scene: &[&dyn Shape], depth: i64) -> Color {
        if depth == 0 {
            return self.background_color();
        }
        let may_collision = self.find_collision(scene);

        match may_collision {
            Some(collision) => {
                let new_color: Color = collision.0.color(self);
                match collision.0.bounce(self) {
                    Some(ray) => new_color.blend(&ray._project_ray(scene, depth - 1)),
                    None => new_color,
                }
                // eprintln!("{} + {} => {}", new_color, color_until_now, ret);
                // handle recursion here 0.5 * diffusion_ray._project_ray(scene, depth - 1)
            }
            None => self.background_color(),
        }
    }

    fn background_color(&self) -> Color {
        let t = 0.5 * (self.direction().normalize().y + 1.0);

        Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }
}
