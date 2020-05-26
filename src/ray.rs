use crate::collision::Collision;
use crate::shapes::shape::Shape;
use crate::Color;
use nalgebra::Vector3;

const T_MIN: f64 = 0.001;
const T_MAX: f64 = 100_000.0;

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

    fn find_collision<'a>(&self, scene: &[&'a (dyn Shape + 'a)]) -> Option<Collision<'a>> {
        let mut maybe_collision: Option<Collision> = None;
        for shape in scene {
            let maybe_new_collision = shape.collide(self, T_MIN, T_MAX);

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

    fn _project_ray(&self, scene: &[&dyn Shape], depth: i64) -> Color {
        if depth == 0 {
            return self.background_color();
        }
        let may_collision = self.find_collision(scene);

        match may_collision {
            Some(collision) => {
                let color_until_now = collision.bounce(self)._project_ray(scene, depth - 1);
                let new_color = collision.color(self);
                let ret = Color::new(
                    new_color.x * color_until_now.x,
                    new_color.y * color_until_now.y,
                    new_color.z * color_until_now.z,
                );
                // eprintln!("{} + {} => {}", new_color, color_until_now, ret);
                ret
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
