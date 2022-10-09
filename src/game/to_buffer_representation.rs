use bytemuck::{Pod, Zeroable};

pub trait ToBufferRepresentation {
    fn to_bits(&self) -> &[u8];
}


// implement the trait for Vec<T> that are Pod, Zeroable, Copy and Clone
impl<T> ToBufferRepresentation for Vec<T>
    where
        T: Pod + Zeroable + Copy + Clone,
{
    fn to_bits(&self) -> &[u8] {
        bytemuck::cast_slice(self.as_slice())
    }
}

