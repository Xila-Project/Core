use core::mem::{size_of, transmute};
use std::mem::align_of;

use lvgl::DisplayRefresh;

use crate::{Area_type, Color_type, Point_type};

#[repr(transparent)]
pub struct Screen_write_data_type {
    Area: Area_type,
    Buffer: &[Color_type],
}

impl Screen_write_data_type {
    pub fn New(Area: Area_type, Buffer: &[Color_type]) -> Self {
        Self { Area, Buffer }
    }
}

impl AsRef<[u8]> for Screen_write_data_type {
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
}

impl TryFrom<&[u8]> for &Screen_write_data_type {
    type Error = ();

    fn try_from(Value: &[u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Screen_write_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % align_of::<Screen_write_data_type>() != 0 {
            return Err(());
        }

        Ok(unsafe { *(Value.as_ptr() as *const Self) })
    }
}

impl Screen_write_data_type {
    pub fn Get_area(&self) -> Area_type {
        Area_type::New(
            Point_type::New(self.Inner.area.x1, self.Inner.area.y1),
            Point_type::New(self.Inner.area.x2, self.Inner.area.y2),
        )
    }

    pub fn Get_buffer(&self) -> &[Color_type; Buffer_size] {
        unsafe { transmute(&self.Inner.colors) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Screen_read_data_type(Point_type);

impl Default for Screen_read_data_type {
    fn default() -> Self {
        Self(Point_type::New(0, 0))
    }
}

impl Screen_read_data_type {
    pub fn Get_resolution(&self) -> Point_type {
        self.0
    }

    pub fn Set_resolution(&mut self, Value: Point_type) {
        self.0 = Value;
    }
}

impl AsMut<[u8]> for Screen_read_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}

impl TryFrom<&mut [u8]> for &mut Screen_read_data_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Screen_read_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % align_of::<Screen_read_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(Value.as_mut_ptr()) })
    }
}
