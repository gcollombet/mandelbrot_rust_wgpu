use wgpu::BufferUsages;
use crate::game::game_state::GameState;
use crate::game::{GameBuffer, Mandelbrot};
use crate::game::Game;
use winit::dpi::PhysicalSize;

use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use crate::game::engine::Engine;

pub struct MandelbrotState {
    mandelbrot: Mandelbrot,
    zoom_speed: f32,
    move_speed: (f32, f32),
}

impl GameState for MandelbrotState {
    fn update(&mut self, engine: &mut Engine, delta_time: f32) {
        let zoom = self.mandelbrot.zoom();
        if self.zoom_speed != 1.0 {
            self.mandelbrot.set_zoom(
                zoom
                * (
                    1.0
                    - (self.zoom_speed * delta_time)
                )
            );
            self.mandelbrot.must_redraw = 0;
        }
        self.mandelbrot.set_maximum_iterations(
            ((1.0
                + (1.0 / zoom)
                .log(2.1)
                .clamp(0.0, 200.0))
                * 100.0) as u32,
        );
        self.mandelbrot.update(delta_time);
        engine.replace_buffer(
            GameBuffer::Mandelbrot as usize,
            BufferUsages::UNIFORM,
            bytemuck::cast_slice(&[self.mandelbrot.get_shader_representation()]),
        );
        engine.replace_buffer(
            GameBuffer::MandelbrotOrbitPointSuite as usize,
            BufferUsages::STORAGE,
            bytemuck::cast_slice(&self.mandelbrot.orbit_point_suite),
        );
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
