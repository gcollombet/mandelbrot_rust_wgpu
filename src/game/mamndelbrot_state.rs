use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::{Deref, Div};
use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferBindingType, BufferUsages, ShaderStages};
use winit::dpi::PhysicalSize;
use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

use to_buffer_representation_derive::ToBufferRepresentation;

use crate::game::{GameBuffer, MandelbrotEngine};
use crate::game::engine::Engine;
use crate::game::Game;
use crate::game::game_state::GameState;
use crate::game::mandelbrot::MandelbrotData;
use crate::game::to_buffer_representation::ToBufferRepresentation;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Pod, Zeroable, ToBufferRepresentation)]
pub struct LastRenderedMandelbrot {
    pub  center_delta: [f32; 2],
    pub  zoom: f32,
    _padding: u32,
}

pub struct MandelbrotState {
    mandelbrot: MandelbrotEngine,
    previous_mandelbrot: MandelbrotEngine,
    mandelbrot_iteration_texture: Rc<RefCell<Vec<f32>>>,
    previous_mandelbrot_iteration_texture: Rc<RefCell<Vec<f32>>>,
    mandelbrot_z_texture: Rc<RefCell<Vec<[f32; 2]>>>,
    zoom_speed: f32,
    zoom_acceleration: f32,
    move_speed: (f32, f32),
    size: PhysicalSize<u32>,
    mouse_position: (isize, isize),
    mouse_left_button_pressed: bool,
    mouse_right_button_pressed: bool,
}

impl GameState for MandelbrotState {

    fn update(&mut self, engine: &mut Engine, delta_time: f32) {
        let epsilon = 0.001;
        self.zoom_acceleration *= 0.05_f32.powf(delta_time);
        if self.zoom_acceleration.abs() < epsilon * 100.0 {
            self.zoom_acceleration = 0.0;
        }
        if self.zoom_speed != 0.0 || self.zoom_acceleration != 0.0 {
            self.mandelbrot.set_zoom(
                self.mandelbrot.zoom()
                    * (1.0 - ((self.zoom_speed + self.zoom_acceleration) * delta_time)),
            );
        }
        self.mandelbrot.set_maximum_iterations(
            ((1.0 + (1.0 / self.mandelbrot.zoom()).log(2.1).clamp(0.0, 100.0)) * 10000.0) as u32,
        );
        self.move_speed.0 *= 0.05_f32.powf(delta_time);
        self.move_speed.1 *= 0.05_f32.powf(delta_time);
        if self.move_speed.0.abs() < epsilon {
            // define a variable that contain value that is randomly 0.01 or -0.01
            let random = if rand::random() { 0.01 } else { -0.01 };
            self.move_speed.0 = 0.0;
        }
        if self.move_speed.1.abs() < epsilon {
            let random = if rand::random() { 0.01 } else { -0.01 };
            self.move_speed.1 = 0.0;
        }
        // if move speed > 0 then move by move speed
        self.mandelbrot.data.deref().borrow_mut().move_by(self.move_speed);
        self.mandelbrot.update(delta_time);
        engine.update_buffer(GameBuffer::Mandelbrot as usize);
        engine.update_buffer(GameBuffer::PreviousMandelbrot as usize);
        engine.update_buffer(GameBuffer::MandelbrotOrbitPointSuite as usize);
        // engine.update_buffer(GameBuffer::MandelbrotIterationTexturePrevious as usize);
        self.previous_mandelbrot.data.deref().borrow_mut().from(&self.mandelbrot.data.deref().borrow());
        // engine.update_buffer(GameBuffer::MandelbrotIterationTexture as usize);
    }

    fn input(&mut self, event: &Event<()>, engine: &mut Engine) {
        if let Event::WindowEvent { ref event, .. } = event {
            match event {
                WindowEvent::Resized(physical_size) => {
                    self.mandelbrot
                        .resize(physical_size.width, physical_size.height);
                    self.mandelbrot_iteration_texture
                        .deref()
                        .borrow_mut()
                        .resize((physical_size.width * physical_size.height) as usize, -2.0);
                    self.previous_mandelbrot_iteration_texture
                        .deref()
                        .borrow_mut()
                        .resize((physical_size.width * physical_size.height) as usize, -2.0);
                    engine.update_buffer(GameBuffer::MandelbrotIterationTexture as usize);
                    engine.update_buffer(GameBuffer::MandelbrotIterationTexturePrevious as usize);
                    self.size = *physical_size;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    let new_inner_size = **new_inner_size;
                    self.mandelbrot
                        .resize(new_inner_size.width, new_inner_size.height);
                    self.mandelbrot_iteration_texture
                        .deref()
                        .borrow_mut()
                        .resize((new_inner_size.width * new_inner_size.height) as usize, -2.0);
                    self.previous_mandelbrot_iteration_texture
                        .deref()
                        .borrow_mut()
                        .resize((new_inner_size.width * new_inner_size.height) as usize, -2.0);
                    engine.update_buffer(GameBuffer::MandelbrotIterationTexture as usize);
                    engine.update_buffer(GameBuffer::MandelbrotIterationTexturePrevious as usize);
                    self.size = new_inner_size;
                }
                // when the mouse scrolls,
                // update the mandelbrot shader zoom
                // by a magnitude of 1.1 or 0.9
                // depending on the direction of the scroll wheel.
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if *y > 0.0 {
                            self.zoom_acceleration += 2.0;
                        } else {
                            self.zoom_acceleration -= 2.0;
                        }
                        // self.mandelbrot.zoom_in(zoom_factor);
                    }
                    MouseScrollDelta::PixelDelta(_) => {}
                },
                // When the arrow keys are pressed or zqsd keys, update the mandelbrot shader coordinates.
                WindowEvent::KeyboardInput { input, .. } => {
                    // detect if keyboard is in french or english
                    if input.state == ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            let movement = 0.010;
                            // if movement is < epsilon then set it to 0.0
                            // let movement = if movement < f32::EPSILON { f32::EPSILON } else { movement };
                            match keycode {
                                // space
                                VirtualKeyCode::Space => {
                                    self.zoom_speed = 0.0;
                                    self.zoom_acceleration = 0.0;
                                }
                                // return
                                VirtualKeyCode::Return => {
                                    self.mandelbrot.data.deref().borrow_mut().reset();
                                }
                                // page up
                                VirtualKeyCode::PageUp => {
                                    self.mandelbrot.data.deref().borrow_mut().color_palette_scale *= 1.1;
                                }
                                // page down
                                VirtualKeyCode::PageDown => {
                                    let value = self.mandelbrot.data.deref().borrow().color_palette_scale.div(1.1).max(0.1);
                                    self.mandelbrot.data.deref().borrow_mut().color_palette_scale =value;
                                }
                                // add
                                VirtualKeyCode::NumpadAdd => {
                                    if self.zoom_speed < 0.0 {
                                        self.zoom_speed /= 1.1;
                                        if self.zoom_speed > -0.1 {
                                            self.zoom_speed = 0.1;
                                        }
                                    } else {
                                        if self.zoom_speed < 0.1 {
                                            self.zoom_speed = 0.5;
                                        }
                                        self.zoom_speed *= 1.1;
                                    }
                                }
                                // subtract
                                VirtualKeyCode::NumpadSubtract => {
                                    if self.zoom_speed < 0.0 {
                                        if self.zoom_speed > -0.1 {
                                            self.zoom_speed = 0.1;
                                        }
                                        self.zoom_speed *= 1.1;
                                    } else {
                                        self.zoom_speed /= 1.1;
                                        if self.zoom_speed < 0.1 {
                                            self.zoom_speed = -0.5;
                                        }
                                    }
                                }
                                // group similar keys together
                                VirtualKeyCode::Left | VirtualKeyCode::Q => {
                                    self.move_speed.0 -= movement;
                                }
                                VirtualKeyCode::Right | VirtualKeyCode::D => {
                                    self.move_speed.0 += movement;
                                }
                                VirtualKeyCode::Up | VirtualKeyCode::Z => {
                                    self.move_speed.1 += movement;
                                }
                                VirtualKeyCode::Down | VirtualKeyCode::S => {
                                    self.move_speed.1 -= movement;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // factorize the mouse MouseInput event
                WindowEvent::MouseInput { state, button, .. } => {
                    if *state == ElementState::Pressed {
                        match button {
                            MouseButton::Left => {
                                self.mouse_position.0 = 0;
                                self.mouse_position.1 = 0;
                                self.mouse_left_button_pressed = true;
                            }
                            MouseButton::Right => {
                                self.mouse_right_button_pressed = true;
                            }
                            _ => {}
                        }
                    } else {
                        match button {
                            MouseButton::Left => {
                                self.mouse_left_button_pressed = false;
                            }
                            MouseButton::Right => {
                                self.mouse_right_button_pressed = false;
                            }
                            _ => {}
                        }
                    }
                }
                // update the mandelbrot shader coordinates when the mouse is moved.
                WindowEvent::CursorMoved { position, .. } => {
                    if self.mouse_left_button_pressed {
                        if self.mouse_position.0 == 0 && self.mouse_position.1 == 0 {
                            self.mouse_position = (position.x as isize, position.y as isize);
                        }
                        self.mandelbrot.data.deref().borrow_mut().move_by_pixel(
                            position.x as isize - self.mouse_position.0,
                            position.y as isize - self.mouse_position.1,
                            self.size.width,
                            self.size.height,
                        );
                    }
                    self.mouse_position.0 = position.x as isize;
                    self.mouse_position.1 = position.y as isize;
                    // if the left mouse button is pressed
                    if self.mouse_right_button_pressed {
                        // update the mandelbrot shader coordinates
                        self.mandelbrot.center_orbit_at(
                            self.mouse_position.0,
                            self.mouse_position.1,
                            self.size.width,
                            self.size.height,
                        );
                    }
                }
                _ => {}
            }
        };
    }
}

impl MandelbrotState {
    // new
    pub fn new(size: PhysicalSize<u32>, engine: &mut Engine) -> Self {
        let mandelbrot = MandelbrotEngine::new(100, size.width, size.height);
        let previous_mandelbrot = MandelbrotEngine::new(100, size.width, size.height);
        let mandelbrot_iteration_texture =
            Rc::new(RefCell::new(vec![-2.0; (size.width * size.height) as usize]));
        // create a buffer to store the previous mandelbrot texture
        let mandelbrot_iteration_texture_previous =
            Rc::new(RefCell::new(vec![-2.0; (size.width * size.height) as usize]));
        // create a buffer to store the z complex (a tuple of two float values) of the mandelbrot
        let mandelbrot_z_texture = Rc::new(RefCell::new(vec![
            [0.0, 0.0];
            (size.width * size.height) as usize
        ]));
        engine.add_buffer(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            BufferBindingType::Uniform,
            ShaderStages::FRAGMENT,
            mandelbrot.data.clone(),
        );
        engine.add_buffer(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            BufferBindingType::Uniform,
            ShaderStages::FRAGMENT,
            previous_mandelbrot.data.clone(),
        );
        engine.add_buffer(
            BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            BufferBindingType::Storage {
                read_only: false,
            },
            ShaderStages::FRAGMENT,
            mandelbrot_iteration_texture.clone());
        engine.add_buffer(
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            BufferBindingType::Storage {
                read_only: false,
            },
            ShaderStages::FRAGMENT,
            mandelbrot_iteration_texture_previous.clone(),
        );
        engine.add_buffer(
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            BufferBindingType::Storage {
                read_only: false,
            },
            ShaderStages::FRAGMENT,
                          mandelbrot_z_texture.clone()
        );
        engine.add_buffer(
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            BufferBindingType::Storage {
                read_only: false,
            },
            ShaderStages::FRAGMENT,
            mandelbrot.orbit_point_suite.clone()
        );
        engine.add_buffer(
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            BufferBindingType::Storage {
                read_only: false,
            },
            ShaderStages::FRAGMENT,
            Rc::new(RefCell::new(LastRenderedMandelbrot {
                zoom: 1.0,
                center_delta: [0.0, 0.0],
                _padding: 0
            }))
        );
        Self {
            mandelbrot,
            previous_mandelbrot,
            mandelbrot_iteration_texture,
            previous_mandelbrot_iteration_texture: mandelbrot_iteration_texture_previous,
            mandelbrot_z_texture,
            zoom_speed: 0.5,
            zoom_acceleration: 0.0,
            move_speed: (0.0, 0.0),
            size,
            mouse_position: (0, 0),
            mouse_left_button_pressed: false,
            mouse_right_button_pressed: false,
        }
    }
}
