use std::cell::RefCell;
use std::rc::Rc;
use wgpu::{BindGroupDescriptor, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType, BufferUsages, Device, Queue, ShaderStages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::game::to_buffer_representation::ToBufferRepresentation;


// create a struct with a name and a   BindGroupLayoutEntry





// A struct called BindedBuffer with a buffer, a bind group, and a bind group layout
pub struct BindBuffer {
    pub data: Rc<RefCell<dyn ToBufferRepresentation>>,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}



// implement new for BindedBuffer
impl BindBuffer {


    pub fn update(&mut self, queue: &Queue) {
        queue.write_buffer(
            &self.buffer,
            0,
            self.data.borrow().to_bits(),
        );
    }

    // create a new BindedBuffer
    pub fn new(
        device: &Device,
        usage: BufferUsages,
        data: Rc<RefCell<dyn ToBufferRepresentation>>,
    ) -> Self {
        let buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: data.borrow().to_bits(),
                usage: usage | BufferUsages::COPY_DST,
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
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            buffer.as_entire_buffer_binding()
                        ),
                    }
                ],
            }
        );
        Self {
            data,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

}