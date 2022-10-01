use wgpu::{BindGroupDescriptor, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsages, Device, Queue, ShaderStages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

// A struct called BindedBuffer with a buffer, a bind group, and a bind group layout
pub struct BindBuffer {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

// implement new for BindedBuffer
impl BindBuffer {
    pub fn new(
        device: &Device,
        usage: BufferUsages,
        data: &[u8],
    ) -> Self {
        let buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: data,
                usage,
            }
        );
        let bind_buffer_type: BufferBindingType;
        match usage {
            BufferUsages::UNIFORM => {
                bind_buffer_type = BufferBindingType::Uniform;
            }
            BufferUsages::STORAGE => {
                bind_buffer_type = BufferBindingType::Storage {read_only: false};
            },
            _ => {
                panic!("Unsupported buffer usage");
            }
        }
        let bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: bind_buffer_type,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );
        let bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            buffer.as_entire_buffer_binding()
                        ),
                    }
                ],
                label: None,
            }
        );
        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    // pub fn new_uniform_buffer(
    //     device: &Device,
    //     data: &[u8],
    // ) -> Self {
    //     let buffer = device.create_buffer_init(
    //         &BufferInitDescriptor {
    //             label: None,
    //             contents: data,
    //             usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    //         }
    //     );
    //     let bind_group_layout = device.create_bind_group_layout(
    //         &BindGroupLayoutDescriptor {
    //             label: None,
    //             entries: &[
    //                 BindGroupLayoutEntry {
    //                     binding: 0,
    //                     visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
    //                     ty: BindingType::Buffer {
    //                         ty: BufferBindingType::Uniform,
    //                         has_dynamic_offset: false,
    //                         min_binding_size: None,
    //                     },
    //                     count: None,
    //                 }
    //             ],
    //         }
    //     );
    //     Self::new(device, buffer, bind_group_layout)
    // }
    //
    //
    //
    // pub fn new_storage_buffer(
    //     device: &Device,
    //     queue: &Queue,
    //     data: &[u8],
    // ) -> Self {
    //     // let buffer = device.create_buffer(
    //     //     &wgpu::BufferDescriptor {
    //     //         label: None,
    //     //         size: data.len() as BufferAddress,
    //     //         usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    //     //         mapped_at_creation: false,
    //     //     }
    //     // );
    //     // queue.write_buffer(
    //     //     &buffer,
    //     //     0,
    //     //     data,
    //     // );
    //     let buffer = device.create_buffer_init(
    //         &BufferInitDescriptor {
    //             label: None,
    //             contents: data,
    //             usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    //         }
    //     );
    //     let bind_group_layout = device.create_bind_group_layout(
    //         &BindGroupLayoutDescriptor {
    //             label: None,
    //             entries: &[
    //                 BindGroupLayoutEntry {
    //                     binding: 0,
    //                     visibility: ShaderStages::FRAGMENT,
    //                     ty: BindingType::Buffer {
    //                         ty: BufferBindingType::Storage { read_only: false },
    //                         has_dynamic_offset: false,
    //                         min_binding_size: None,
    //                     },
    //                     count: None,
    //                 },
    //             ],
    //         }
    //     );
    //     return Self::new(device, buffer, bind_group_layout);
    // }
}