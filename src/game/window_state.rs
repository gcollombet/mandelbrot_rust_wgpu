use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use crate::game::Game;
use crate::game::game_state::GameState;

pub struct WindowState {
    size: PhysicalSize<u32>,
    is_fullscreen: bool,
    mouse_position: (isize, isize),
    mouse_left_button_pressed: bool,
    mouse_right_button_pressed: bool,
}

impl WindowState {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            size,
            is_fullscreen: false,
            mouse_position: (0, 0),
            mouse_left_button_pressed: false,
            mouse_right_button_pressed: false,
        }
    }
}

impl GameState for WindowState {
    fn update(&mut self, game: &mut Game) {
        todo!()
    }

    fn input(&mut self, event: &Event<()>, game: &mut Game) {
        match event {
            Event::WindowEvent {
                ref event, ..
            } => match event {
                WindowEvent::Resized(physical_size) => {
                    self.size = *physical_size;
                }
                _ => {}
            },
            _ => {}
        };
    }
}