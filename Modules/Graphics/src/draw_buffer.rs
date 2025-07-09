use alloc::{vec, vec::Vec};

use crate::{Color_RGB565_type, Color_type, Point_type, Rendering_color_type};

#[repr(transparent)]
pub struct Buffer_type {
    buffer: Vec<Rendering_color_type>,
}

impl AsRef<[Color_type]> for Buffer_type {
    fn as_ref(&self) -> &[Color_type] {
        unsafe {
            let buffer = self.buffer.as_ptr() as *const Color_type;
            core::slice::from_raw_parts(buffer, self.buffer.len())
        }
    }
}

impl Buffer_type {
    pub fn new_from_resolution(resolution: &Point_type) -> Self {
        let buffer_size = get_minimal_buffer_size(resolution);

        Self::new(buffer_size)
    }

    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![Color_RGB565_type::new(0, 0, 0); buffer_size],
        }
    }
}

pub const fn get_recommended_buffer_size(resolution: &Point_type) -> usize {
    resolution.get_x() as usize * resolution.get_y() as usize
}

pub const fn get_minimal_buffer_size(resolution: &Point_type) -> usize {
    if resolution.get_x() < resolution.get_y() {
        resolution.get_y() as usize * resolution.get_y() as usize / 10
    } else {
        resolution.get_x() as usize * resolution.get_x() as usize / 10
    }
}
