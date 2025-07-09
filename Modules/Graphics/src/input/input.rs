use alloc::boxed::Box;

use core::ffi::c_void;

use file_system::Device_type;

use crate::{lvgl, Result_type};

use super::{Binding_callback_function, Input_type_type, User_data_type};

pub struct Input_type {
    #[allow(dead_code)]
    input_device: *mut lvgl::lv_indev_t,
}

impl Drop for Input_type {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(
                lvgl::lv_indev_get_user_data(self.input_device) as *mut User_data_type
            );

            lvgl::lv_indev_delete(self.input_device);

            // User_data will be dropped here.
        }
    }
}

unsafe impl Send for Input_type {}

unsafe impl Sync for Input_type {}

impl Input_type {
    pub fn new(device: Device_type, Type: Input_type_type) -> Result_type<Self> {
        // User_data is a pinned box, so it's ownership can be transferred to LVGL and will not move or dropper until the Input_device is dropped.
        let User_data = Box::new(User_data_type { device });

        let Input_device = unsafe {
            let input_device = lvgl::lv_indev_create();
            lvgl::lv_indev_set_type(input_device, Type.into());
            lvgl::lv_indev_set_read_cb(input_device, Some(Binding_callback_function));
            lvgl::lv_indev_set_user_data(input_device, Box::into_raw(User_data) as *mut c_void);

            if Type == Input_type_type::Keypad {
                let group = lvgl::lv_group_get_default();

                lvgl::lv_indev_set_group(input_device, group);
            }

            input_device
        };

        Ok(Self {
            input_device: Input_device,
        })
    }
}
