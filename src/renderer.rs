use nalgebra::Vector3;

pub type Color = Vector3<f64>;

struct Size {
    width: usize,
    height: usize,
}

pub struct RendererPPM {
    pixels: Vec<Color>,
    size: Size,
    samples_per_pixel: i64,
}

impl RendererPPM {
    pub fn new(height: usize, width: usize, samples_per_pixel: i64) -> Self {
        let count = width * height;
        let mut v = Vec::with_capacity(count);
        v.resize_with(count, || Vector3::new(0.0,0.0,0.0));
        Self {
            pixels: v,
            size: Size{width, height},
            samples_per_pixel
        }
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y * self.size.width + x] = color;
    }
    pub fn render(&self) {
        println!("P3\n{} {} \n255", self.size.width, self.size.height);
        for y in (0..self.size.height).rev() {
            for x in 0..self.size.width {
                let color = self.pixels[y * self.size.width + x];
                
                let scale = 1.0 / self.samples_per_pixel as f64;
                let ir = (255.999 * (color[0] * scale).clamp(0.0, 1.0).sqrt()) as i64;
                let ig = (255.999 * (color[1] * scale).clamp(0.0, 1.0).sqrt()) as i64;
                let ib = (255.999 * (color[2] * scale).clamp(0.0, 1.0).sqrt()) as i64;
                println!("{} {} {}", ir, ig, ib);
            }
        }
    }
}
