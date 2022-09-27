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

use winit::window::Window;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Mandelbrot {
    seed: f32,
    zoom: f32,
    x: f32,
    y: f32,
    maximum_iterations: u32,
    width: u32,
    height: u32,
    mu: f32,
    is_rendered: u32
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            seed: 0.0,
            zoom: 1.0,
            x: 0.0,
            y: 0.0,
            maximum_iterations: 1000,
            width: 0,
            height: 0,
            mu: 10000.0,
            is_rendered: 0
        }
    }
}

impl Mandelbrot {

    // a function that zoom in the mandelbrot set by a given factor.
    // the function take as parameters :
    // - the zoom factor
    // - the x and y coordinates of the mouse
    // - the width and height of the window
    // The function compute the normalized vector of the mouse position relatively to the window center in a variable called "normalized_mouse_vector"
    // Then it multiply the mouse_vector by the zoom factor in a new variable called "scaled_mouse_vector"
    // Then it add the "scaled_mouse_vector" to the current x and y coordinates of the mandelbrot set
    // Then it multiply the "scaled_mouse_vector" by the zoom factor in a new variable called "zoomed_scaled_mouse_vector"
    // Then it add the "zoomed_scaled_mouse_vector" times -1 to the current x and y coordinates of the mandelbrot set
    pub fn zoom_in(&mut self, zoom_factor: f32, mouse_x: f32, mouse_y: f32, window_width: u32, window_height: u32) {
        let normalized_mouse_vector = (
            (mouse_x - (window_width as f32 / 2.0)) / (window_width as f32 / 2.0),
            (mouse_y - (window_height as f32 / 2.0)) / (window_height as f32 / 2.0)
        );
        let scaled_mouse_vector = (
            normalized_mouse_vector.0 * self.zoom,
            normalized_mouse_vector.1 * self.zoom
        );
        self.x += scaled_mouse_vector.0;
        self.y -= scaled_mouse_vector.1;
        let zoomed_scaled_mouse_vector = (
            scaled_mouse_vector.0 * zoom_factor,
            scaled_mouse_vector.1 * zoom_factor
        );
        self.x -= zoomed_scaled_mouse_vector.0;
        self.y += zoomed_scaled_mouse_vector.1;
        self.zoom *= zoom_factor;
        self.is_rendered = 0;
    }

}

// implement new for MandelbrotShader, without zoom, x, y, mu
impl Mandelbrot {
    fn new(maximum_iterations: u32, width: u32, height: u32) -> Self {
        Self {
            maximum_iterations,
            width,
            height,
            ..Default::default()
        }
    }
}

// A struct called BindedBuffer with a buffer, a bind group, and a bind group layout
struct BindedBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

// implement new for BindedBuffer
impl BindedBuffer {
    fn new(
        device: &wgpu::Device,
        buffer: wgpu::Buffer,
        bind_group_layout: wgpu::BindGroupLayout
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
            label: None,
        });
        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
}

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
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] =
// A square
    &[
        // first triangle
        Vertex { position: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [1.0, 1.0, 0.0], color: [0.0, 0.0, 1.0] },
        Vertex { position: [-1.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },
        // second triangle
        Vertex { position: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [1.0, -1.0, 0.0], color: [0.0, 0.0, 1.0] },
        Vertex { position: [1.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },
    ];


impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],
        }
    }
}

impl Game {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
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
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        // windows height and width
        let mandelbrot_shader = Mandelbrot::new(1000, size.width, size.height );
        let mandelbrot_shader_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mandelbrot Buffer"),
                contents: bytemuck::cast_slice(&[mandelbrot_shader]),
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
        queue.write_buffer(&mandelbrot_texture_buffer, 0,mandelbrot_texture_data.as_slice());

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
            mandelbrot: mandelbrot_shader,
            mandelbrot_uniform_buffer: mandelbrot_shader_buffer,
            mandelbrot_uniform_bind_group: mandelbrot_shader_bind_group,
            mandelbrot_texture_buffer,
            mandelbrot_texture_bind_group,
            last_screen_update: Instant::now(),
            mouse_position: (0.0, 0.0),
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
            self.queue.write_buffer(&self.mandelbrot_texture_buffer, 0,mandelbrot_texture_data.as_slice());
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
        self.mandelbrot.zoom *= 0.998;
        self.mandelbrot.is_rendered = 0;
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
    window.set_title("Turtle soccer");

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
            println!("New frame: {}ms since last frame, {} frames rendered so far", time_since_last_screen_update.as_millis(), state.mandelbrot.seed);
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
            // update the mandelbrot shader coordinates when the mouse is moved.
            WindowEvent::CursorMoved { position, .. } => {
                state.mouse_position.0 = position.x as f32;
                state.mouse_position.1 = position.y as f32;
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
            // when the mouse scrolls, update the mandelbrot shader zoom by a magnitude of 1.1 or 0.9 depending on the direction of the scroll wheel.
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        let mut zoom_factor = 1.1;
                        if *y > 0.0 {
                            zoom_factor = 0.9;
                        }
                        state.mandelbrot.zoom_in(
                            zoom_factor,
                            state.mouse_position.0,
                            state.mouse_position.1,
                            state.size.width,
                            state.size.height
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

