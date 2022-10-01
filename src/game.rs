mod mandelbrot;
mod engine;

use std::time::{Duration, Instant};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};
use mandelbrot::Mandelbrot;
use engine::{
    Engine,
    bind_buffer::BindBuffer
};

struct Game {
    size: winit::dpi::PhysicalSize<u32>,
    is_fullscreen: bool,
    engine: Engine,
    mouse_position: (f32, f32),
    mouse_left_button_pressed: bool,
    mouse_right_button_pressed: bool,
    zoom_speed: f32,
    move_speed: (f32, f32),
    mandelbrot: Mandelbrot,
    last_screen_update: Instant,
}

impl Game {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let mandelbrot = Mandelbrot::new(
            1000,
            size.width,
            size.height,
        );
        let mut engine = Engine::new(window).await;
        engine.add_uniform_buffer(
            bytemuck::cast_slice(&[mandelbrot])
        );
        let mandelbrot_texture_data = vec![0u8; (size.width * size.height * 4) as usize];
        engine.add_storage_buffer(
            mandelbrot_texture_data.as_slice()
        );
        engine.create_pipeline();
        Self {
            engine,
            size,
            mandelbrot,
            last_screen_update: Instant::now(),
            mouse_position: (0.0, 0.0),
            is_fullscreen: false,
            zoom_speed: 0.995,
            mouse_left_button_pressed: false,
            mouse_right_button_pressed: false,
            move_speed: (0.0, 0.0),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.engine.resize(new_size);
            self.mandelbrot.height = self.size.height;
            self.mandelbrot.width = self.size.width;
            self.mandelbrot.must_redraw = 0;
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // add one to the mandelbrot seed
        self.mandelbrot.generation += 1.0;
        self.engine.update();

        if self.zoom_speed != 1.0 {
            self.mandelbrot.zoom *= self.zoom_speed;
            self.mandelbrot.must_redraw = 0;
        }
        let last_max_iterations = self.mandelbrot.maximum_iterations;
        // mandelbrot max iterations is log_10 of the inverse of the zoom
        self.mandelbrot.maximum_iterations = (1.0 + (1.0 / self.mandelbrot.zoom).log2().clamp(0.0, 100.0)) as u32 * 200 + 100;
        // print max iterations to the console if it has changed
        if self.mandelbrot.maximum_iterations != last_max_iterations {
            println!("max iterations: {}", self.mandelbrot.maximum_iterations);
        }
        // update the mandelbrot shader buffer
        self.engine.queue.write_buffer(
            &self.engine.uniform_buffers[0].buffer,
            0,
            bytemuck::cast_slice(&[self.mandelbrot]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.engine.render().expect("TODO: panic message");
        if self.mandelbrot.must_redraw == 0 {
            self.mandelbrot.must_redraw = 1;
        }
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Mandelbrot");
    // window.set_fullscreen(Some(Fullscreen::Borderless(None)));
    let mut state = Game::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id)
        if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // this is the time between screen updates
            let time_between_screen_updates = Duration::from_millis(1000 / 144);
            // this is the time between the last screen update and now
            let time_since_last_screen_update = Instant::now() - state.last_screen_update;
            // this is the time until the next screen update

            // if the time since the last screen update is greater than the time between screen updates
            if time_since_last_screen_update < time_between_screen_updates {
                // if the time since the last screen update is less than the time between screen updates
                // then we need to wait until the next screen update
                // so we set the time until the next screen update
                let time_until_next_screen_update = time_between_screen_updates - time_since_last_screen_update;
                // and we set the control flow to wait until the next screen update
                *control_flow = ControlFlow::WaitUntil(Instant::now() + time_until_next_screen_update);
            }
            // update the last screen update time
            state.last_screen_update = Instant::now();
            // request a redraw
            window.request_redraw();
            // print new frame to the console with the time since the last screen update and the total count of frames rendered so far
            // println!("New frame: {}ms since last frame, {} frames rendered so far", time_since_last_screen_update.as_millis(), state.mandelbrot.seed);
        }
        Event::WindowEvent {
            ref event,
            window_id,
        }
        if window_id == window.id() && !state.input(event) => match event {
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                state.resize(**new_inner_size);
            }
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
                state.is_fullscreen = !state.is_fullscreen;
                if state.is_fullscreen {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                } else {
                    window.set_fullscreen(None);
                }
            }
            // when the key page up is pressed
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::PageUp),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                state.mandelbrot.color_palette_scale = 0.01 + state.mandelbrot.color_palette_scale as f32 * 1.1;
            }
            // when the key page down is pressed
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::PageDown),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                state.mandelbrot.color_palette_scale = 0.01 + state.mandelbrot.color_palette_scale as f32 / 1.1;
            }
            // when the + key is pressed increase the the zoom speed by 1.1
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::NumpadAdd),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                state.zoom_speed /= 1.0005;
            }
            // when the - key is pressed decrease the the zoom speed by 1.1
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::NumpadSubtract),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                state.zoom_speed *= 1.0005;
            }
            // when the escape key is pressed exit the program
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // when the enter key is pressed reset the zoom speed to 0.0
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Return),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                state.mandelbrot.reset();
            }
            // when the space bar is pressed
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                // reset the mandelbrot
                state.zoom_speed = 1.0;
            }
            // update the mandelbrot shader coordinates when the mouse is moved.
            WindowEvent::CursorMoved { position, .. } => {
                if state.mouse_left_button_pressed {
                    if state.mouse_position.0 == 0.0 && state.mouse_position.1 == 0.0 {
                        state.mouse_position = (position.x as f32, position.y as f32);
                    }
                    state.mandelbrot.move_by_pixel(
                        position.x as f32 - state.mouse_position.0,
                        position.y as f32 - state.mouse_position.1,
                        state.size.width,
                        state.size.height,
                    );
                }
                state.mouse_position.0 = position.x as f32;
                state.mouse_position.1 = position.y as f32;
                // if the left mouse button is pressed
                if state.mouse_right_button_pressed {
                    // update the mandelbrot shader coordinates
                    state.mandelbrot.center_orbit_at(
                        state.mouse_position.0,
                        state.mouse_position.1,
                        state.size.width,
                        state.size.height,
                    );
                }
            }
            // when zero is pressed
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Numpad0),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                state.mandelbrot.center_to_orbit();
            }
            // When the arrow keys are pressed or zqsd keys, update the mandelbrot shader coordinates.
            WindowEvent::KeyboardInput { input, .. } => {
                // detect if keyboard is in french or english
                if let Some(keycode) = input.virtual_keycode {
                    let movement = 0.025 * state.mandelbrot.zoom;
                    // if movement is < epsilon then set it to 0.0
                    // let movement = if movement < f32::EPSILON { f32::EPSILON } else { movement };
                    match keycode {
                        // group similar keys together
                        VirtualKeyCode::Left | VirtualKeyCode::Q => {
                            state.mandelbrot.move_by((-movement, 0.0));
                        }
                        VirtualKeyCode::Right | VirtualKeyCode::D => {
                            state.mandelbrot.move_by((movement, 0.0));
                        }
                        VirtualKeyCode::Up | VirtualKeyCode::Z => {
                            state.mandelbrot.move_by((0.0, movement));
                        }
                        VirtualKeyCode::Down | VirtualKeyCode::S => {
                            state.mandelbrot.move_by((0.0, -movement));
                        }
                        _ => {}
                    }
                }
            }
            // when the mouse is left clicked
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                state.mouse_position.0 = 0.0;
                state.mouse_position.1 = 0.0;
                // set the mouse position to the mandelbrot shader coordinates
                // state.mandelbrot.center_at(
                //     state.mouse_position.0,
                //     state.mouse_position.1,
                //     state.size.width,
                //     state.size.height,
                // );
                state.mouse_left_button_pressed = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                state.mouse_right_button_pressed = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Right,
                ..
            } => {
                state.mouse_right_button_pressed = false;
            }
            // when the mouse is left released
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                state.mouse_left_button_pressed = false;
            }
            // when the mouse scrolls, update the mandelbrot shader zoom by a magnitude of 1.1 or 0.9 depending on the direction of the scroll wheel.
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        let mut zoom_factor = 1.1;
                        if *y > 0.0 {
                            zoom_factor = 0.9;
                        }
                        state.mandelbrot.zoom_in(
                            zoom_factor
                        );
                    }
                    MouseScrollDelta::PixelDelta(_) => {}
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}

