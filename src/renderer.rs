use nalgebra::Vector3;
use std::borrow::{Borrow, BorrowMut};
use std::io;
use std::io::{BufWriter, Write};

pub type Color = Vector3<f64>;

struct Size {
    width: usize,
    height: usize,
}

pub struct RendererPPM {
    pixels: Vec<Color>,
    size: Size,
}

impl RendererPPM {
    pub fn new(height: usize, width: usize) -> Self {
        let count = width * height;
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, || Vector3::new(0.0, 0.0, 0.0));
        Self {
            pixels: v,
            size: Size { width, height },
        }
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y * self.size.width + x] = color;
    }
    pub fn render(&self) {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        let header_lines = format!("P3\n{} {} \n255\n", self.size.width, self.size.height);
        writer.write_all(header_lines.as_bytes());

        for y in (0..self.size.height).rev() {
            for x in 0..self.size.width {
                let color = self.pixels[y * self.size.width + x];

                let ir = (255.999 * color[0]) as i64;
                let ig = (255.999 * color[1]) as i64;
                let ib = (255.999 * color[2]) as i64;
                writer.write_all(format!("{} {} {}\n", ir, ig, ib).as_bytes());
            }
        }
    }
}
