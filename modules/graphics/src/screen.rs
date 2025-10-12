use core::mem::{size_of, transmute};

use crate::{Area, Point, RenderingColor};

pub struct ScreenWriteData<'a> {
    area: Area,
    buffer: &'a [RenderingColor],
}

impl<'a> ScreenWriteData<'a> {
    pub fn new(area: Area, buffer: &'a [RenderingColor]) -> Self {
        Self { area, buffer }
    }
}

impl AsRef<[u8]> for ScreenWriteData<'_> {
    fn as_ref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
}

impl TryFrom<&[u8]> for &ScreenWriteData<'_> {
    type Error = ();

    /// This function is used to convert a buffer of bytes to a struct.
    ///
    /// # Arguments
    ///
    /// * `Value` - A buffer of bytes.
    ///
    /// # Errors
    ///
    /// This function will return an error if the size of the buffer is not the same as the size of the struct.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer.
    ///
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<ScreenWriteData>() {
            return Err(());
        }

        if !(value.as_ptr() as usize).is_multiple_of(align_of::<ScreenWriteData>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*const u8, Self>(value.as_ptr()) })
    }
}

impl ScreenWriteData<'_> {
    pub fn get_area(&self) -> Area {
        self.area
    }

    pub fn get_buffer(&self) -> &[RenderingColor] {
        self.buffer
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ScreenReadData(Point);

impl Default for ScreenReadData {
    fn default() -> Self {
        Self(Point::new(0, 0))
    }
}

impl ScreenReadData {
    pub fn get_resolution(&self) -> Point {
        self.0
    }

    pub fn set_resolution(&mut self, value: Point) {
        self.0 = value;
    }
}

impl AsMut<[u8]> for ScreenReadData {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}

impl TryFrom<&mut [u8]> for &mut ScreenReadData {
    type Error = ();

    fn try_from(value: &mut [u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<ScreenReadData>() {
            return Err(());
        }
        if !(value.as_ptr() as usize).is_multiple_of(align_of::<ScreenReadData>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(value.as_mut_ptr()) })
    }
}
