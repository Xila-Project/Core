use std::{mem::MaybeUninit, pin::Pin};

use super::lvgl;

use crate::{Color_type, Point_type};

#[repr(transparent)]
pub struct Buffer_type<const Buffer_size: usize> {
    Buffer: Pin<Box<[MaybeUninit<Color_type>; Buffer_size]>>,
}

impl<const Buffer_size: usize> Default for Buffer_type<Buffer_size> {
    fn default() -> Self {
        Self {
            Buffer: Box::pin([MaybeUninit::uninit(); Buffer_size]),
        }
    }
}

impl From<Buffer_type> for *mut Color_type {
    fn from(Buffer: Buffer_type) -> *mut Color_type {
        Box::into_raw(Buffer.Buffer) as *mut Color_type
    }
}

pub const fn Get_recommended_buffer_size(Resolution: &Point_type) -> usize {
    if Resolution.Get_x() < Resolution.Get_y() {
        Resolution.Get_y() as usize * Resolution.Get_y() as usize / 10
    } else {
        Resolution.Get_x() as usize * Resolution.Get_x() as usize / 10
    }
}
