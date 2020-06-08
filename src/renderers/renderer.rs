use nalgebra::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Vector3<f64>> for PixelColor {
    fn from(vector3: Vector3<f64>) -> Self {
        Self {
            r: vector3.x as u8,
            g: vector3.y as u8,
            b: vector3.z as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub struct PixelPosition {
    pub x: usize,
    pub y: usize,
}

pub type PixelAccessor = dyn Fn(PixelPosition, PixelColor) + Send;

pub trait Renderer {
    fn new(dimensions: Dimensions) -> Self;

    fn pixel_accessor(&mut self) -> Box<PixelAccessor>;

    // fn render(&self);

    fn start_rendering(&mut self);
}
