use core::mem::{size_of, transmute};

use crate::{Area_type, Point_type, Rendering_color_type};

pub struct Screen_write_data_type<'a> {
    area: Area_type,
    buffer: &'a [Rendering_color_type],
}

impl<'a> Screen_write_data_type<'a> {
    pub fn new(area: Area_type, buffer: &'a [Rendering_color_type]) -> Self {
        Self { area, buffer }
    }
}

impl AsRef<[u8]> for Screen_write_data_type<'_> {
    fn as_ref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
}

impl TryFrom<&[u8]> for &Screen_write_data_type<'_> {
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
        if value.len() != size_of::<Screen_write_data_type>() {
            return Err(());
        }

        if value.as_ptr() as usize % align_of::<Screen_write_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*const u8, Self>(value.as_ptr()) })
    }
}

impl Screen_write_data_type<'_> {
    pub fn get_area(&self) -> Area_type {
        self.area
    }

    pub fn get_buffer(&self) -> &[Rendering_color_type] {
        self.buffer
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Screen_read_data_type(Point_type);

impl Default for Screen_read_data_type {
    fn default() -> Self {
        Self(Point_type::new(0, 0))
    }
}

impl Screen_read_data_type {
    pub fn get_resolution(&self) -> Point_type {
        self.0
    }

    pub fn set_resolution(&mut self, value: Point_type) {
        self.0 = value;
    }
}

impl AsMut<[u8]> for Screen_read_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}

impl TryFrom<&mut [u8]> for &mut Screen_read_data_type {
    type Error = ();

    fn try_from(value: &mut [u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<Screen_read_data_type>() {
            return Err(());
        }
        if value.as_ptr() as usize % align_of::<Screen_read_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(value.as_mut_ptr()) })
    }
}
