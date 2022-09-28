
// A struct called BindedBuffer with a buffer, a bind group, and a bind group layout
pub struct BindBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

// implement new for BindedBuffer
impl BindBuffer {
    pub fn new(
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