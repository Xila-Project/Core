use std::ffi::c_void;

//use lvgl::input_device::{pointer, InputDriver};
use File_system::File_type;

use crate::{Display::Display_type, Pointer_data_type, Result_type};

use super::lvgl;

struct User_data_type<'a> {
    pub File: File_type<'a>,
}

pub struct Input_type {
    #[allow(dead_code)]
    Input_device: *mut lvgl_rust_sys::lv_indev_t,
}

impl Drop for Input_type {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(
                lvgl::lv_indev_get_user_data(self.Input_device) as *mut User_data_type
            );

            lvgl::lv_indev_delete(self.Input_device);

            // User_data will be dropped here.
        }
    }
}

unsafe impl Send for Input_type {}

unsafe impl Sync for Input_type {}

/// This function is called by LVGL when it needs to read input data.
///
/// # Safety
///
/// This function may dereference a raw pointer.
unsafe extern "C" fn Binding_callback_function(
    Input_device: *mut lvgl::lv_indev_t,
    Data: *mut lvgl::lv_indev_data_t,
) {
    let User_data = unsafe { lvgl::lv_indev_get_user_data(Input_device) as *mut User_data_type };

    let File = &(*User_data).File;

    let mut Pointer_data = Pointer_data_type::default();

    File.Read(Pointer_data.as_mut())
        .expect("Error reading from input device");

    unsafe {
        *Data = Pointer_data.into();
    }
}

impl Input_type {
    pub fn New<const Buffer_size: usize>(
        File: File_type,
        _: &Display_type<Buffer_size>,
    ) -> Result_type<Self> {
        // User_data is a pinned box, so it's ownership can be transferred to LVGL and will not move or dropper until the Input_device is dropped.
        let User_data = Box::new(User_data_type { File });

        let Input_device = unsafe {
            let Input_device = lvgl::lv_indev_create();
            lvgl::lv_indev_set_type(Input_device, lvgl::lv_indev_type_t_LV_INDEV_TYPE_POINTER);
            lvgl::lv_indev_set_read_cb(Input_device, Some(Binding_callback_function));
            lvgl::lv_indev_set_user_data(Input_device, Box::into_raw(User_data) as *mut c_void);

            Input_device
        };

        Ok(Self { Input_device })
    }
}
