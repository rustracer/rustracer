use lazy_static::lazy_static;

use std::sync::Mutex;

use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};

lazy_static! {
    /// Getter to a random(inclusive: f64, exclusive: f64)
    pub static ref RNG: Mutex<Option<Box<dyn FnMut(f64, f64) -> f64 + Send>>> = Mutex::new(None);
}

pub fn init_RNG<R: FnMut(f64, f64) -> f64 + Send + 'static>(get_random_f64: R) {
    *RNG.lock().unwrap() = Some(Box::new(get_random_f64));
}

pub fn rand_range_f64(start: f64, stop: f64) -> f64 {
    // Will panic if RNG is not set properly!
    RNG.lock().unwrap().as_mut().as_mut().unwrap()(start, stop)
}

/// Ugly port from https://docs.rs/crate/rand/0.7.3/source/src/seq/mod.rs
pub fn shuffle<T>(slice: &mut [T]) {
    for i in (1..slice.len()).rev() {
        // invariant: elements with index > i have been locked in place.
        slice.swap(i, gen_index(i + 1));
    }
}

fn gen_index(ubound: usize) -> usize {
    rand_range_f64(0.0, (ubound) as f64) as usize
}