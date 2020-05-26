extern crate lazy_static;

use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

lazy_static! {
    static ref RNG: Mutex<SmallRng> = Mutex::new(SmallRng::from_entropy());
}

pub fn rand_range_f64(start: f64, stop: f64) -> f64 {
    RNG.lock().unwrap().gen_range(start, stop)
}
