use bytemuck::{Pod, Zeroable};
use to_buffer_representation_derive::ToBufferRepresentation;
use crate::game::to_buffer_representation::ToBufferRepresentation;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Pod, Zeroable, ToBufferRepresentation)]
pub struct MandelbrotDot {
    // the value of z
    pub z: [f32; 2],
    // the value of the derivative of z
    pub derivative: [f32; 2],
    // the number of iterations to reach the maximum value
    pub iterations: i32,
    pub reference_iteration: i32,
}

// implement default for MandelbrotDot
impl Default for MandelbrotDot {
    fn default() -> Self {
        Self {
            z: [0.0, 0.0],
            derivative: [1.0, 0.0],
            iterations: 0,
            reference_iteration: 0,
        }
    }
}

// implement new for MandelbrotDot
impl MandelbrotDot {
    pub fn new() -> Self {
        Self::default()
    }
}