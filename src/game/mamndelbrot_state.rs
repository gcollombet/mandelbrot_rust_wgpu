use crate::game::game_state::GameState;
use crate::game::Mandelbrot;
use crate::game::Game;
use winit::dpi::PhysicalSize;

use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

pub struct MandelbrotState {
    mandelbrot: Mandelbrot,
    zoom_speed: f32,
    move_speed: (f32, f32),
}

impl GameState for MandelbrotState {
    fn update(&mut self, game: &mut Game) {
        todo!()
    }

    fn input(&mut self, event: &Event<()>, game: &mut Game) {
        match event {
            Event::WindowEvent {
                ref event, ..
            } => match event {
                WindowEvent::Resized(physical_size) => {
                    self.mandelbrot.resize(physical_size.width, physical_size.height);
                }
                _ => {}
            },
            _ => {}
        };
    }
}

impl MandelbrotState {
    // new
    pub fn new(size: PhysicalSize<u32>) -> Self {
        let mandelbrot = Mandelbrot::new(10, size.width, size.height);
        Self {
            mandelbrot,
            zoom_speed: 0.9,
            move_speed: (0.0, 0.0),
        }
    }
}
