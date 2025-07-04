use core::mem::{size_of, transmute};

use crate::{Area_type, Point_type, Rendering_color_type};

pub struct Screen_write_data_type<'a> {
    Area: Area_type,
    Buffer: &'a [Rendering_color_type],
}

impl<'a> Screen_write_data_type<'a> {
    pub fn New(Area: Area_type, Buffer: &'a [Rendering_color_type]) -> Self {
        Self { Area, Buffer }
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
    fn try_from(Value: &[u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Screen_write_data_type>() {
            return Err(());
        }

        if Value.as_ptr() as usize % align_of::<Screen_write_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*const u8, Self>(Value.as_ptr()) })
    }
}

impl Screen_write_data_type<'_> {
    pub fn Get_area(&self) -> Area_type {
        self.Area
    }

    pub fn Get_buffer(&self) -> &[Rendering_color_type] {
        self.Buffer
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
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
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
