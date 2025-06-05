use alloc::boxed::Box;

use core::ffi::c_void;

use File_system::Device_type;

use crate::{Result_type, LVGL};

use super::{Binding_callback_function, Input_type_type, User_data_type};

pub struct Input_type {
    #[allow(dead_code)]
    Input_device: *mut LVGL::lv_indev_t,
}

impl Drop for Input_type {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(
                LVGL::lv_indev_get_user_data(self.Input_device) as *mut User_data_type
            );

            LVGL::lv_indev_delete(self.Input_device);

            // User_data will be dropped here.
        }
    }
}

unsafe impl Send for Input_type {}

unsafe impl Sync for Input_type {}

impl Input_type {
    pub fn New(Device: Device_type, Type: Input_type_type) -> Result_type<Self> {
        // User_data is a pinned box, so it's ownership can be transferred to LVGL and will not move or dropper until the Input_device is dropped.
        let User_data = Box::new(User_data_type { Device });

        let Input_device = unsafe {
            let Input_device = LVGL::lv_indev_create();
            LVGL::lv_indev_set_type(Input_device, Type.into());
            LVGL::lv_indev_set_read_cb(Input_device, Some(Binding_callback_function));
            LVGL::lv_indev_set_user_data(Input_device, Box::into_raw(User_data) as *mut c_void);

            if Type == Input_type_type::Keypad {
                let Group = LVGL::lv_group_get_default();

                LVGL::lv_indev_set_group(Input_device, Group);
            }

            Input_device
        };

        Ok(Self { Input_device })
    }
}
