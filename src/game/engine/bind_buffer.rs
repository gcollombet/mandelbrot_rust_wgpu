use crate::game::to_buffer_representation::ToBufferRepresentation;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType,
    BufferUsages, Device, Queue, ShaderStages,
};

// create a struct name PipelineBuffer
// a name,
// a BindGroupLayoutEntry,
// a BindGroupEntry,
// a Buffer,
// a Queue,
// and a field named data, with the buffer data as  Rc<RefCell<dyn ToBufferRepresentation>>

pub struct PipelineBuffer {
    pub name: String,
    pub bind_group_layout_entry: BindGroupLayoutEntry,
    // pub bind_group_entry: BindGroupEntry<'a>,
    data: Rc<RefCell<dyn ToBufferRepresentation>>,
}

// implement PipelineBuffer for PipelineBuffer struct
impl PipelineBuffer {
    // create a new function that takes
    // a device,
    // a queue,
    // a name,
    // a data,
    // a usage,
    // a shader stage,
    // a binding
    // and a binding type as parameters
    // and returns a PipelineBuffer
    pub fn new(
        device: &Device,
        name: String,
        data: Rc<RefCell<dyn ToBufferRepresentation>>,
        usage: BufferUsages,
        shader_stage: ShaderStages,
        binding: u32,
        binding_type: BindingType,
    ) -> Self {
        let contents = data.borrow().to_bits();
        // create a buffer with the device and the queue
        // and the data from the data parameter
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&name),
            contents,
            usage,
        });
        // create a bind_group_layout_entry with the name, the shader stage and the binding type
        let bind_group_layout_entry = BindGroupLayoutEntry {
            binding,
            visibility: shader_stage,
            ty: binding_type,
            count: None,
        };
        // create a bind_group_entry with the binding and the buffer binding type
        // let bind_group_entry = BindGroupEntry {
        //     binding,
        //     resource: buffer.as_entire_binding(),
        // };
        // return a PipelineBuffer with the name, the bind_group_layout_entry, the bind_group_entry, the buffer and the data
        Self {
            name,
            bind_group_layout_entry,
            // bind_group_entry,
            data,
        }
    }

    // create a function named update that updates the buffer

    pub fn update(&mut self) {
        // // get the buffer from the resource using if let
        // if let BindingResource::Buffer(buffer_binding) = &self.bind_group_entry.resource.borrow() {
        //     // get the buffer from the buffer binding
        //     let buffer = buffer_binding.buffer;
        // update the buffer with the queue and the bits
        // self.queue
        //     .write_buffer(&buffer, 0, self.data.borrow().to_bits());
        // }
    }
}
