
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub coordinate: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    // first triangle
    Vertex { position: [-1.0, -1.0, 0.0], coordinate: [-1.0, -1.0] },
    Vertex { position: [1.0, 1.0, 0.0], coordinate: [1.0, 1.0] },
    Vertex { position: [-1.0, 1.0, 0.0], coordinate: [-1.0, 1.0] },
    // second triangle
    Vertex { position: [-1.0, -1.0, 0.0], coordinate: [-1.0, -1.0] },
    Vertex { position: [1.0, -1.0, 0.0], coordinate: [1.0, -1.0] },
    Vertex { position: [1.0, 1.0, 0.0], coordinate: [1.0, 1.0] },
];