mod engine;
mod mandelbrot;

use std::time::{Duration, Instant};
use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::window::{Window, WindowBuilder};

use engine::Engine;
use mandelbrot::Mandelbrot;
use wgpu::BufferUsages;
use winit::event_loop::ControlFlow;

// create an enum with the name of the different buffer
enum GameBuffer {
    Mandelbrot = 0,
    MandelbrotTexture = 1,
    MandelbrotOrbitPointSuite = 2,
}

pub struct WindowState {
    size: winit::dpi::PhysicalSize<u32>,
    is_fullscreen: bool,
    mouse_position: (isize, isize),
    mouse_left_button_pressed: bool,
    mouse_right_button_pressed: bool,
}

pub struct MandelbrotState {
    mandelbrot: Mandelbrot,
    zoom_speed: f32,
    move_speed: (f32, f32),
}

pub struct Game {
    window_state: WindowState,
    mandelbrot_state: MandelbrotState,
    engine: Engine,
    mandelbrot_texture: Vec<f32>,
    last_screen_update: Instant,
    last_frame_time: Duration,
}

impl Game {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
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
            engine,
            mandelbrot_state: MandelbrotState {
                mandelbrot,
                zoom_speed: 0.9,
                move_speed: (0.0, 0.0),
            },
            last_screen_update: Instant::now(),
            window_state: WindowState {
                size,
                is_fullscreen: false,
                mouse_position: (0, 0),
                mouse_left_button_pressed: false,
                mouse_right_button_pressed: false,
            },
            mandelbrot_texture: mandelbrot_texture_data,
            last_frame_time: Duration::from_secs_f32(1.0 / 120.0),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_state.size = new_size;
            self.engine.resize(new_size);
            self.mandelbrot_state.mandelbrot.height = self.window_state.size.height;
            self.mandelbrot_state.mandelbrot.width = self.window_state.size.width;
            self.mandelbrot_state.mandelbrot.must_redraw = 0;
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
            Event::RedrawRequested(window_id) if window_id == self.window_state.window.id() => {
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
