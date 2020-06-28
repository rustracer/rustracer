use raytracer_core::PixelAccessor;

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub(crate) width: usize,
    pub(crate) height: usize,
}


pub trait Renderer {
    fn new(dimensions: Dimensions) -> Self;

    fn pixel_accessor(&mut self, weight: f32) -> Box<PixelAccessor>;

    // fn render(&self);

    fn start_rendering(&mut self);
}
