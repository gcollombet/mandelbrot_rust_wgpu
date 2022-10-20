use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferUsages, Device, Queue, ShaderStages,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::game::to_buffer_representation::ToBufferRepresentation;

// create a struct to hold a bind group layout entry, a bind group entry, and a buffer

pub struct BindGroupBufferEntry {
    pub bind_group_layout_entry: BindGroupLayoutEntry,
    pub buffer: Buffer,
    length: usize,
    usage: BufferUsages,
    pub data: Rc<RefCell<dyn ToBufferRepresentation>>,
}

// implement new for BindGroupBufferEntry
impl BindGroupBufferEntry {

    // create that return a bing group entry
    pub fn bind_group_entry(&self) -> BindGroupEntry {
        let buffer = &self.buffer;
        let binding = self.bind_group_layout_entry.binding;
        BindGroupEntry {
            binding,
            resource: buffer.as_entire_binding(),
        }
    }

    // length of the buffer
    pub fn length(&self) -> usize {
        self.length
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) {
        let data: &RefCell<dyn ToBufferRepresentation> = self.data.borrow();
        let data = data.borrow();
        let contents = data.to_bits();
        if self.length != contents.len() {
            self.length = contents.len();
            self.buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Buffer"),
                contents,
                usage: self.usage,
            });
        }
        queue.write_buffer(&self.buffer, 0, contents);
    }


    // create a new BindGroupBufferEntry
    pub fn new(
        device: &Device,
        binding: u32,
        visibility: ShaderStages,
        usage: BufferUsages,
        buffer_binding_type: BufferBindingType,
        data: Rc<RefCell<dyn ToBufferRepresentation>>,
    ) -> Self { ;
        // create a buffer from the data
        let _data: &RefCell<dyn ToBufferRepresentation> = data.borrow();
        let length = _data.borrow().to_bits().len();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Buffer"),
            contents: _data.borrow().to_bits(),
            usage,
        });
        // borrow the data
        let bind_group_layout_entry = BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: buffer_binding_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        Self {
            bind_group_layout_entry,
            length,
            usage,
            buffer,
            data,
        }
    }
}
