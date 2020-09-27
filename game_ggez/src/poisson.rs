use rand::Rng;

pub struct Poisson {
}

impl Poisson {
    pub fn new() -> Self {
        Poisson{}
    }
    pub fn compute_new_position(&self, existing_points: &Vec<(f64, f64)>, near_point: &(f64, f64), radius: f64, nb_attempts: u32, mut random: impl Rng)  -> Option<(f64, f64)> {
        let seed = random.next_u64() as f64 / std::u64::MAX as f64;
        let radius_squared = radius * radius;
        let radius_plus_epsilon = radius + 0.01;
        for attempt_amount in 0..nb_attempts {
            // TODO: generate a random point around given point
            let theta  = 2_f64 * std::f64::consts::PI * (seed as f64 + 1f64*attempt_amount as f64/(nb_attempts as f64));
            let test_point = (near_point.0 + radius_plus_epsilon * theta.cos(), near_point.1 + radius_plus_epsilon * theta.sin());
            let mut is_correct = true;
            for existing_point in existing_points.iter() {
                if distance_squared(existing_point, &test_point).sqrt() < radius_plus_epsilon{
                    is_correct = false;
                    break;
                }
            }
            if is_correct {
                return Some(test_point);
            }
        }
        None
    }
}

pub fn distance_squared(p1: &(f64, f64), p2: &(f64, f64)) -> f64 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    dx * dx + dy * dy
}
