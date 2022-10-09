use std::cell::RefCell;
use std::rc::Rc;
use wgpu::BufferUsages;
use crate::game::game_state::GameState;
use crate::game::{GameBuffer, Mandelbrot};
use crate::game::Game;
use winit::dpi::PhysicalSize;

use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use crate::game::engine::Engine;

#[derive(Debug)]
pub struct MandelbrotState {
    mandelbrot: Mandelbrot,
    mandelbrot_texture: Rc<RefCell<Vec<f32>>>,
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
        }
        self.mandelbrot.set_maximum_iterations(
            ((1.0
                + (1.0 / zoom)
                .log(2.1)
                .clamp(0.0, 200.0))
                * 100.0) as u32,
        );
        self.mandelbrot.update(delta_time);
        engine.update_buffer(GameBuffer::Mandelbrot as usize);
        engine.update_buffer(GameBuffer::MandelbrotOrbitPointSuite as usize);
    }

    fn input(&mut self, event: &Event<()>, engine: &mut Engine) {
        match event {
            Event::WindowEvent {
                ref event, ..
            } => match event {
                WindowEvent::Resized(physical_size) => {
                    self.mandelbrot.resize(physical_size.width, physical_size.height);
                    // self.mandelbrot_texture.borrow_mut().resize(
                    //     (physical_size.width * physical_size.height) as usize,
                    //     0.0,
                    // );
                    engine.update_buffer(GameBuffer::MandelbrotTexture as usize);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    let new_inner_size = **new_inner_size;
                    self.mandelbrot.resize( new_inner_size.width, new_inner_size.height);
                    // self.mandelbrot_texture.borrow_mut().resize(
                    //     (new_inner_size.width * new_inner_size.height) as usize,
                    //     0.0,
                    // );
                    engine.update_buffer(GameBuffer::MandelbrotTexture as usize);
                }
                _ => {}
            },
            _ => {}
        };
    }
}

impl MandelbrotState {
    // new
    pub fn new(size: PhysicalSize<u32>, engine: &mut Engine) -> Self {
        let mandelbrot = Mandelbrot::new(100, size.width, size.height);
        let mandelbrot_texture = Rc::new(RefCell::new(vec![0.0; (size.width * size.height) as usize]));
        engine.add_buffer(
            BufferUsages::UNIFORM,
            mandelbrot.shader_representation.clone(),
        );
        engine.add_buffer(
            BufferUsages::STORAGE,
            mandelbrot_texture.clone(),
        );
        engine.add_buffer(
            BufferUsages::STORAGE,
            mandelbrot.orbit_point_suite.clone(),
        );
        Self {
            mandelbrot,
            mandelbrot_texture,
            zoom_speed: 0.2,
            move_speed: (0.0, 0.0),
        }
    }
}
