mod engine;
mod mandelbrot;
mod game_state;
mod window_state;
mod mamndelbrot_state;

use std::time::{Duration, Instant};
use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::window::{Window, WindowBuilder};

use engine::Engine;
use mandelbrot::Mandelbrot;
use wgpu::BufferUsages;
use winit::event_loop::ControlFlow;
use game_state::GameState;
use window_state::WindowState;
use mamndelbrot_state::MandelbrotState;

// create an enum with the name of the different buffer
enum GameBuffer {
    Mandelbrot = 0,
    MandelbrotTexture = 1,
    MandelbrotOrbitPointSuite = 2,
}

pub struct Game<'a> {
    window: &'a Window,
    window_state: WindowState,
    mandelbrot_state: MandelbrotState,
    engine: Engine,
    mandelbrot_texture: Vec<f32>,
    last_screen_update: Instant,
    last_frame_time: Duration,
}

impl<'a> Game<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Game<'a> {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let mut mandelbrot = Mandelbrot::new(10, size.width, size.height);
        let mut engine = Engine::new(window).await;
        engine.add_buffer(
            BufferUsages::UNIFORM,
            bytemuck::cast_slice(&[mandelbrot.get_shader_representation()]),
        );
        let mandelbrot_texture_data = vec![0f32; (size.width * size.height) as usize];
        let mandelbrot_z_data = vec![0f32; (size.width * size.height) as usize];
        engine.add_buffer(
            BufferUsages::STORAGE,
            bytemuck::cast_slice(&mandelbrot_texture_data),
        );
        engine.add_buffer(
            BufferUsages::STORAGE,
            bytemuck::cast_slice(&mandelbrot.orbit_point_suite),
        );
        engine.add_buffer(
            BufferUsages::STORAGE,
            bytemuck::cast_slice(&mandelbrot_z_data),
        );
        engine.create_pipeline();
        Self {
            window,
            engine,
            mandelbrot_state: MandelbrotState::new(size),
            last_screen_update: Instant::now(),
            window_state: WindowState::new(size),
            mandelbrot_texture: mandelbrot_texture_data,
            last_frame_time: Duration::from_secs_f32(1.0 / 120.0),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.engine.resize(new_size);
            // self.mandelbrot_state.mandelbrot.must_redraw = 0;
            self.mandelbrot_texture.resize(
                (self.window_state.size.width * self.window_state.size.height) as usize,
                0.0,
            );
            self.engine.replace_buffer(
                GameBuffer::MandelbrotTexture as usize,
                BufferUsages::STORAGE,
                bytemuck::cast_slice(&self.mandelbrot_texture),
            );
        }
    }

    pub fn input(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::RedrawRequested(window_id) => {
                self.update();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.window_state.size),
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
            _ => {}
        }
    }

    pub fn update(&mut self) {
        let time_since_last_screen_update = Instant::now() - self.last_screen_update;
        // add one to the mandelbrot seed
        self.mandelbrot_state.mandelbrot.generation += self.last_frame_time.as_secs_f32();
        if self.mandelbrot_state.zoom_speed != 1.0 {
            self.mandelbrot_state.mandelbrot.zoom *=
                1.0 - (self.mandelbrot_state.zoom_speed * self.last_frame_time.as_secs_f32());
            self.mandelbrot_state.mandelbrot.must_redraw = 0;
        }
        let last_max_iterations = self.mandelbrot_state.mandelbrot.maximum_iterations();
        // mandelbrot max iterations is log_10 of the inverse of the zoom
        self.mandelbrot_state.mandelbrot.set_maximum_iterations(
            ((1.0
                + (1.0 / self.mandelbrot_state.mandelbrot.zoom)
                .log(2.1)
                .clamp(0.0, 200.0))
                * 100.0) as u32,
        );
        // print max iterations to the console if it has changed
        if self.mandelbrot_state.mandelbrot.maximum_iterations() != last_max_iterations {
            println!(
                "max iterations: {}",
                self.mandelbrot_state.mandelbrot.maximum_iterations()
            );
        } else {
            self.mandelbrot_state.mandelbrot.update();
        }
        self.engine.replace_buffer(
            GameBuffer::Mandelbrot as usize,
            BufferUsages::UNIFORM,
            bytemuck::cast_slice(&[self.mandelbrot_state.mandelbrot.get_shader_representation()]),
        );
        self.engine.replace_buffer(
            GameBuffer::MandelbrotOrbitPointSuite as usize,
            BufferUsages::STORAGE,
            bytemuck::cast_slice(&self.mandelbrot_state.mandelbrot.orbit_point_suite),
        );
        if self.mandelbrot_state.mandelbrot.must_redraw == 0 {
            self.mandelbrot_state.mandelbrot.must_redraw = 1;
        }
        self.engine.update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.engine.render().expect("TODO: panic message");
        Ok(())
    }
}
