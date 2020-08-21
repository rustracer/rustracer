use std::sync::mpsc::Sender;

use crate::PixelRendererCommunicator;

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub(crate) width: usize,
    pub(crate) height: usize,
}

pub struct MoveCommand {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
pub struct RotateCommand {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub enum Command {
    Move(MoveCommand),
    Rotate(RotateCommand),
}

// TODO: this trait is useless for now..
pub trait Renderer {
    fn new(dimensions: Dimensions, tx: Sender<Command>) -> Self;

    fn pixel_accessor(&mut self) -> PixelRendererCommunicator;

    // fn render(&self);

    fn start_rendering(self);
}
