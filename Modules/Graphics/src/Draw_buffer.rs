use std::{mem::MaybeUninit, pin::Pin};

use crate::{Color_type, Point_type};

#[repr(transparent)]
pub struct Buffer_type<const Buffer_size: usize> {
    Buffer: Pin<Box<[MaybeUninit<Color_type>; Buffer_size]>>,
}

impl<const Buffer_size: usize> AsRef<[Color_type]> for Buffer_type<Buffer_size> {
    fn as_ref(&self) -> &[Color_type] {
        unsafe {
            &*(&*self.Buffer as *const [MaybeUninit<Color_type>; Buffer_size]
                as *const [Color_type; Buffer_size])
        }
    }
}

impl<const Buffer_size: usize> Default for Buffer_type<Buffer_size> {
    fn default() -> Self {
        Self {
            Buffer: Pin::new(Box::new([MaybeUninit::uninit(); Buffer_size])),
        }
    }
}

pub const fn Get_recommended_buffer_size(Resolution: &Point_type) -> usize {
    if Resolution.Get_x() < Resolution.Get_y() {
        Resolution.Get_y() as usize * Resolution.Get_y() as usize / 10
    } else {
        Resolution.Get_x() as usize * Resolution.Get_x() as usize / 10
    }
}
