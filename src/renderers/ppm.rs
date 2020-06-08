use crate::renderers::renderer::{Dimensions, PixelColor, Renderer};

#[derive(Debug, Clone)]
pub struct RendererPPM {
    pixels: Vec<PixelColor>,
    dimensions: Dimensions,
}

impl Renderer for RendererPPM {
    fn new(dimensions: Dimensions) -> Self {
        let size = dimensions.width * dimensions.height;
        let black = PixelColor { r: 0, g: 0, b: 0 };
        Self {
            pixels: vec![black; size],
            dimensions,
        }
    }

    fn set_pixel_at(&mut self, x: usize, y: usize, color: PixelColor) {
        self.pixels[y * self.dimensions.width + x] = color;
    }

    fn render(&self) {
        println!(
            "P3\n{} {} \n255",
            self.dimensions.width, self.dimensions.height
        );
        for y in (0..self.dimensions.height).rev() {
            for x in 0..self.dimensions.width {
                let color = self.pixels[y * self.dimensions.width + x];

                println!("{} {} {}", color.r, color.g, color.b);
            }
        }
    }
}
