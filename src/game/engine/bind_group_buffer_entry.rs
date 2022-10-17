use crate::game::to_buffer_representation::ToBufferRepresentation;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferUsages, Device, Queue, ShaderStages,
};

// create a struct to hold a bind group layout entry, a bind group entry, and a buffer

pub struct BindGroupBufferEntry<'a> {
    pub bind_group_layout_entry: BindGroupLayoutEntry,
    pub bind_group_entry: BindGroupEntry<'a>,
    pub buffer: Buffer,
    pub data: Rc<RefCell<dyn ToBufferRepresentation>>,
}

// implement new for BindGroupBufferEntry
impl<'a> BindGroupBufferEntry<'a> {
    pub fn new(
        device: &'a Device,
        binding: u32,
        visibility: ShaderStages,
        usage: BufferUsages,
        buffer_binding_type: BufferBindingType,
        data: Rc<RefCell<dyn ToBufferRepresentation>>,
    ) -> BindGroupBufferEntry<'a> {
        // create a buffer from the data
        let _data: &RefCell<dyn ToBufferRepresentation> = data.borrow();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Buffer"),
            contents: _data.borrow().to_bits(),
            usage,
        });
        let buffer2 = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Buffer"),
            contents: _data.borrow().to_bits(),
            usage,
        });
        let bind_group_entry = BindGroupEntry {
            binding,
            resource: buffer2.as_entire_binding(),
        };
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
            bind_group_entry,
            buffer,
            data,
        }
    }
}
