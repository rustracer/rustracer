use crate::PixelRendererCommunicator;
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub enum Command {
    Up,
    Down,
    Left,
    Right,
}

// TODO: this trait is useless for now..
pub trait Renderer {
    fn new(dimensions: Dimensions, tx: Sender<Command>) -> Self;

    fn pixel_accessor(&mut self) -> PixelRendererCommunicator;

    // fn render(&self);

    fn start_rendering(self);
}
