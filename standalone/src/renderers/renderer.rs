use raytracer_core::PixelRenderer;

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub trait Renderer {
    fn new(dimensions: Dimensions) -> Self;

    fn pixel_accessor(&mut self) -> Box<dyn PixelRenderer>;

    // fn render(&self);

    fn start_rendering(&mut self);
}
