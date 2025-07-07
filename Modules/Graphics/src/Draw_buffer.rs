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
    pub fn new_from_resolution(Resolution: &Point_type) -> Self {
        let buffer_size = Get_minimal_buffer_size(Resolution);

        Self::New(buffer_size)
    }

    pub fn New(Buffer_size: usize) -> Self {
        Self {
            buffer: vec![Color_RGB565_type::new(0, 0, 0); Buffer_size],
        }
    }
}

pub const fn Get_recommended_buffer_size(Resolution: &Point_type) -> usize {
    Resolution.Get_x() as usize * Resolution.Get_y() as usize
}

pub const fn Get_minimal_buffer_size(Resolution: &Point_type) -> usize {
    if Resolution.Get_x() < Resolution.Get_y() {
        Resolution.Get_y() as usize * Resolution.Get_y() as usize / 10
    } else {
        Resolution.Get_x() as usize * Resolution.Get_x() as usize / 10
    }
}
