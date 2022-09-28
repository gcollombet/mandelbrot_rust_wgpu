mod mandelbrot;
mod bind_buffer;
mod vertex;

use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::platform::windows::WindowExtWindows;
use winit::window::{Fullscreen, Window};

use mandelbrot::Mandelbrot;
use bind_buffer::BindBuffer;
use vertex::Vertex;
use vertex::VERTICES;

struct Game {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    // add the mandelbrot shader
    mandelbrot: Mandelbrot,
    // add the mandelbrot shader buffer and bind group
    mandelbrot_uniform_buffer: wgpu::Buffer,
    mandelbrot_uniform_bind_group: wgpu::BindGroup,
    // add the mandelbrot texture and bind group
    mandelbrot_texture_buffer: wgpu::Buffer,
    mandelbrot_texture_bind_group: wgpu::BindGroup,
    // last_screen_update
    last_screen_update: Instant,
    // add the mouse position
    mouse_position: (f32, f32),
    is_fullscreen: bool,
    zoom_speed: f32,
    mouse_left_button_pressed: bool,
}

impl Game {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let mandelbrot = Mandelbrot::new(100000, size.width, size.height);
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();
        let modes = surface.get_supported_modes(&adapter);
        // if modes countain Mailbox, use it, otherwise use FIFO
        let mode = modes.iter()
            .find(|m| **m == wgpu::PresentMode::Mailbox)
            .unwrap_or(&wgpu::PresentMode::Fifo);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: *mode,
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mandelbrot.wgsl").into()),
        });
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        // windows height and width
        let mandelbrot_shader_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mandelbrot Buffer"),
                contents: bytemuck::cast_slice(&[mandelbrot]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // create a simple float buffer of the size of the number of pixels in the window
        let mandelbrot_texture_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Mandelbrot Texture Buffer"),
                size: (size.width * size.height * 4) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::MAP_READ
                    | wgpu::BufferUsages::MAP_WRITE,
                mapped_at_creation: false,
            }
        );
        // initialize the mandelbrot texture buffer
        // create a simple u8 array of the size of the number of pixels in the window
        let mut mandelbrot_texture_data = vec![0u8; (size.width * size.height * 4) as usize];
        queue.write_buffer(&mandelbrot_texture_buffer, 0, mandelbrot_texture_data.as_slice());

        // create a bind group for the mandelbrot texture
        let mandelbrot_texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Mandelbrot Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }
        );
        let mandelbrot_texture_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Mandelbrot Texture Bind Group"),
                layout: &mandelbrot_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: mandelbrot_texture_buffer.as_entire_binding(),
                    },
                ],
            }
        );
        // create a bind group for the mandelbrot shader
        let mandelbrot_shader_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Mandelbrot Shader Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }
        );
        let mandelbrot_shader_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Mandelbrot Shader Bind Group"),
                layout: &mandelbrot_shader_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: mandelbrot_shader_buffer.as_entire_binding(),
                    },
                ],
            }
        );

        // create a render pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &mandelbrot_shader_bind_group_layout,
                    &mandelbrot_texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let num_vertices = VERTICES.len() as u32;
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            mandelbrot,
            mandelbrot_uniform_buffer: mandelbrot_shader_buffer,
            mandelbrot_uniform_bind_group: mandelbrot_shader_bind_group,
            mandelbrot_texture_buffer,
            mandelbrot_texture_bind_group,
            last_screen_update: Instant::now(),
            mouse_position: (0.0, 0.0),
            is_fullscreen: false,
            zoom_speed: 0.995,
            mouse_left_button_pressed: false,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // resize the mandelbrot_texture_buffer to the new size
            // recreate the mandelbrot_texture_buffer
            self.mandelbrot_texture_buffer = self.device.create_buffer(
                &wgpu::BufferDescriptor {
                    label: Some("Mandelbrot Texture Buffer"),
                    size: (self.size.width * self.size.height * 4) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }
            );
            let mut mandelbrot_texture_data = vec![0u8; (self.size.width * self.size.height * 4) as usize];
            self.queue.write_buffer(&self.mandelbrot_texture_buffer, 0, mandelbrot_texture_data.as_slice());
            // recreate the mandelbrot_texture_bind_group and layout
            let mandelbrot_texture_bind_group_layout = self.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Mandelbrot Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            );
            self.mandelbrot_texture_bind_group = self.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Mandelbrot Texture Bind Group"),
                    layout: &mandelbrot_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.mandelbrot_texture_buffer.as_entire_binding(),
                        },
                    ],
                }
            );
            self.mandelbrot.height = self.size.height;
            self.mandelbrot.width = self.size.width;
            self.mandelbrot.is_rendered = 0;
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // add one to the mandelbrot seed
        self.mandelbrot.seed += 1.0;

        if self.zoom_speed != 1.0 {
            self.mandelbrot.zoom *= self.zoom_speed;
            self.mandelbrot.is_rendered = 0;
        }
        let last_max_iterations = self.mandelbrot.maximum_iterations;
        // mandelbrot max iterations is log_10 of the inverse of the zoom
        self.mandelbrot.maximum_iterations = (1.0 + (1.0 / self.mandelbrot.zoom).log2().clamp(0.0, 100.0)) as u32 * 50 + 100;
        // print max iterations to the console if it has changed
        if self.mandelbrot.maximum_iterations != last_max_iterations {
            println!("max iterations: {}", self.mandelbrot.maximum_iterations);
        }
        // update the mandelbrot shader buffer
        self.queue.write_buffer(
            &self.mandelbrot_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.mandelbrot]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.mandelbrot_uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.mandelbrot_texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
            // print the content of the storage buffer mandelbrot texture buffer to the console
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        if self.mandelbrot.is_rendered == 0 {
            self.mandelbrot.is_rendered = 1;
        }
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Mandelbrot");
    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
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
            let time_between_screen_updates = Duration::from_millis(1000 / 60);
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
                state.mandelbrot.color_palette_scale = 1 + (state.mandelbrot.color_palette_scale as f32 * 1.1) as u32;
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
                state.mandelbrot.color_palette_scale = 1 + (state.mandelbrot.color_palette_scale as f32 / 1.1) as u32;
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
                state.mouse_position.0 = position.x as f32;
                state.mouse_position.1 = position.y as f32;
                // if the left mouse button is pressed
                if state.mouse_left_button_pressed {
                    // update the mandelbrot shader coordinates
                    state.mandelbrot.center_at(
                        state.mouse_position.0,
                        state.mouse_position.1,
                        state.size.width,
                        state.size.height,
                    );
                }
            }
            // When the arrow keys are pressed or zqsd keys, update the mandelbrot shader coordinates.
            WindowEvent::KeyboardInput { input, .. } => {
                // detect if keyboard is in french or english
                if let Some(keycode) = input.virtual_keycode {
                    let movement = 0.1 * state.mandelbrot.zoom;
                    match keycode {
                        // group similar keys together
                        VirtualKeyCode::Left | VirtualKeyCode::Q => {
                            state.mandelbrot.x -= movement;
                        }
                        VirtualKeyCode::Right | VirtualKeyCode::D => {
                            state.mandelbrot.x += movement;
                        }
                        VirtualKeyCode::Up | VirtualKeyCode::Z => {
                            state.mandelbrot.y += movement;
                        }
                        VirtualKeyCode::Down | VirtualKeyCode::S => {
                            state.mandelbrot.y -= movement;
                        }
                        _ => {}
                    }
                    state.mandelbrot.is_rendered = 0;
                }
            }
            // when the mouse is left clicked
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                // set the mouse position to the mandelbrot shader coordinates
                state.mandelbrot.center_at(
                    state.mouse_position.0,
                    state.mouse_position.1,
                    state.size.width,
                    state.size.height,
                );
                state.mouse_left_button_pressed = true;
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

