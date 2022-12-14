use std::rc::Rc;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::window::{Fullscreen, Window};

use crate::game::engine::Engine;
use crate::game::Game;
use crate::game::game_state::GameState;

#[derive(Debug)]
pub struct WindowState {
    window: Rc<Window>,
    is_fullscreen: bool,
}

impl WindowState {
    pub fn new(window: Rc<Window>) -> Self {
        Self {
            window,
            is_fullscreen: false,
        }
    }
}

impl GameState for WindowState {
    fn update(&mut self, engine: &mut Engine, delta_time: f32) {
        // engine.resize(self.size);
    }

    fn input(&mut self, event: &Event<()>, engine: &mut Engine) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == self.window.id() => match event {
                // toogle fullscreen on f11
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F11),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    self.is_fullscreen = !self.is_fullscreen;
                    if self.is_fullscreen {
                        self.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        self.window.set_fullscreen(None);
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
}