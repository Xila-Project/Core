use crate::{Color_type, Point_type};

#[repr(transparent)]
pub struct Buffer_type {
    Buffer: Vec<Color_type>,
}

impl AsRef<[Color_type]> for Buffer_type {
    fn as_ref(&self) -> &[Color_type] {
        unsafe {
            let Buffer = self.Buffer.as_ptr() as *const Color_type;
            std::slice::from_raw_parts(Buffer, self.Buffer.len())
        }
    }
}

impl Buffer_type {
    pub fn New_from_resolution(Resolution: &Point_type) -> Self {
        let Buffer_size = Get_minimal_buffer_size(Resolution);

        Self::New(Buffer_size)
    }

    pub fn New(Buffer_size: usize) -> Self {
        Self {
            Buffer: vec![Color_type::New(0, 0, 0); Buffer_size],
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
