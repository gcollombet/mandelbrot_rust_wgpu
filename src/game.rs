use std::borrow::Borrow;
use std::rc::Rc;
use std::time::{Duration, Instant};

use wgpu::BufferUsages;
use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::ControlFlow;
use winit::window::{Window, WindowBuilder};

use engine::Engine;
use game_state::GameState;
use mamndelbrot_state::MandelbrotState;
use mandelbrot::MandelbrotEngine;
use window_state::WindowState;

mod engine;
mod game_state;
mod mamndelbrot_state;
mod mandelbrot;
mod to_buffer_representation;
mod window_state;

// create an enum with the name of the different buffer
enum GameBuffer {
    Mandelbrot = 0,
    PreviousMandelbrot = 1,
    MandelbrotIterationTexture = 2,
    PreviousMandelbrotIterationTexture = 3,
    MandelbrotData = 4,
    PreviousMandelbrotData = 5,
    MandelbrotOrbitPointSuite = 6,
}

pub struct Game {
    window: Rc<Window>,
    window_state: WindowState,
    mandelbrot_state: MandelbrotState,
    engine: Engine,
    last_screen_update: Instant,
    pub last_frame_time: Duration,
}

impl Game {
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    // Creating some of the wgpu types requires async code
    pub async fn new(window: Rc<Window>) -> Self {
        let size = window.inner_size();
        let mut engine = Engine::new(window.borrow()).await;
        let mandelbrot_state = MandelbrotState::new(size, &mut engine);
        engine.create_pipeline();
        Self {
            window: window.clone(),
            engine,
            mandelbrot_state,
            last_screen_update: Instant::now(),
            window_state: WindowState::new(window.clone()),
            last_frame_time: Duration::from_secs_f32(1.0 / 120.0),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.engine.resize(new_size);
        }
    }

    pub fn input(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        self.window_state.input(&event, &mut self.engine);
        self.mandelbrot_state.input(&event, &mut self.engine);
        match event {
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                self.update();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.window.inner_size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // this is the time between screen updates
                let time_between_screen_updates = Duration::from_millis(1000 / 120);
                // this is the time between the last screen update and now
                let time_since_last_screen_update = Instant::now() - self.last_screen_update;
                self.last_frame_time = time_since_last_screen_update;
                self.last_screen_update = Instant::now();
                // this is the time until the next screen update
                // if the time since the last screen update is greater than the time between screen updates
                if time_since_last_screen_update < time_between_screen_updates {
                    // if the time since the last screen update is less than the time between screen updates
                    // then we need to wait until the next screen update
                    // so we set the time until the next screen update
                    let time_until_next_screen_update =
                        time_between_screen_updates - time_since_last_screen_update;
                    // update the last screen update time
                    if time_until_next_screen_update > Duration::from_millis(0) {
                        // and we set the control flow to wait until the next screen update
                        *control_flow =
                            ControlFlow::WaitUntil(Instant::now() + time_until_next_screen_update);
                    }
                }
                // request a redraw
                self.window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(**new_inner_size);
                }
                // when the escape key is pressed exit the program
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
        }
    }

    pub fn update(&mut self) {
        let delta_time = self.last_frame_time.as_secs_f32();
        self.window_state.update(&mut self.engine, delta_time);
        self.mandelbrot_state.update(&mut self.engine, delta_time);
        self.engine.update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.engine.render()
    }
}
