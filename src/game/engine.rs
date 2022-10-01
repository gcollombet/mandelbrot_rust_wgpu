pub mod bind_buffer;
pub mod vertex;

use winit::window::{Fullscreen, Window};
use wgpu::ShaderModule;
use wgpu::util::DeviceExt;
use bind_buffer::BindBuffer;
use crate::game::engine::vertex::{Vertex, VERTICES};

pub struct Engine {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    shader: Option<ShaderModule>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    pub uniform_buffers: Vec<BindBuffer>,
    pub storage_buffers: Vec<BindBuffer>,
    vertex_buffer: wgpu::Buffer,
}


// implement engine for Engine struct whith a new function
impl Engine {
    // the new function takes a window as a parameter
    // and initializes the engine with the window like it is done in Game new function
    // the idea is to refactor the Game new function to use the Engine new function
    pub async fn new(window: &Window) -> Self {
        // create surface
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        // create adapter
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.expect("Impossible to find a GPU!");
        // create device and queue
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
        ).await.expect("Impossible to create device and queue!");
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
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let mut engine = Self {
            surface,
            config,
            queue,
            device,
            render_pipeline: None,
            shader: None,
            uniform_buffers: vec![],
            storage_buffers: vec![],
            vertex_buffer,
        };
        engine
    }

    pub fn update(&mut self) {
        // TODO
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
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
                }
            );
            render_pass.set_pipeline(&self.render_pipeline.as_ref().unwrap());

            // set bind groups from bind buffers with incrementing index
            for (i, bind_buffer) in self.uniform_buffers.iter().enumerate() {
                render_pass.set_bind_group(0 as u32, &bind_buffer.bind_group, &[]);
            }
            for (i, bind_buffer) in self.storage_buffers.iter().enumerate() {
                render_pass.set_bind_group(1 as u32, &bind_buffer.bind_group, &[]);
            }
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn add_storage_buffer(&mut self, data: &[u8]) {
        self.storage_buffers.push(
            BindBuffer::new_storage_buffer(
                &self.device,
                &self.queue,
                data
            )
        );
    }

    pub fn add_uniform_buffer(&mut self, data: &[u8]) {
        self.uniform_buffers.push(
            BindBuffer::new_uniform_buffer(
                &self.device,
                data
            )
        );
    }

    pub fn create_pipeline(&mut self) {
        let shader = self.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/mandelbrot.wgsl").into()
                ),
            });
        // extract a slice of bind group layouts from the uniform and storage buffers
        let bind_group_layouts = self
            .uniform_buffers
            .iter()
            .map(|b| &b.bind_group_layout)
            .chain(self.storage_buffers.iter().map(|b| &b.bind_group_layout))
            .collect::<Vec<_>>();

        // create a render pipeline layout
        let render_pipeline_layout =
            self.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                }
            );
        let render_pipeline = self.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
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
                        format: self.config.format,
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
            }
        );
        self.render_pipeline = Some(render_pipeline);
    }
}
