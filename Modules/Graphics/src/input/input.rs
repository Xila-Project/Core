use alloc::boxed::Box;

use core::ffi::c_void;

use file_system::DeviceType;

use crate::{lvgl, Result};

use super::{binding_callback_function, InputKind, UserDataType};

pub struct InputType {
    #[allow(dead_code)]
    input_device: *mut lvgl::lv_indev_t,
}

impl Drop for InputType {
    fn drop(&mut self) {
        unsafe {
            let _ =
                Box::from_raw(lvgl::lv_indev_get_user_data(self.input_device) as *mut UserDataType);

            lvgl::lv_indev_delete(self.input_device);

            // User_data will be dropped here.
        }
    }
}

unsafe impl Send for InputType {}

unsafe impl Sync for InputType {}

impl InputType {
    pub fn new(device: DeviceType, r#type: InputKind) -> Result<Self> {
        // User_data is a pinned box, so it's ownership can be transferred to LVGL and will not move or dropper until the Input_device is dropped.
        let user_data = Box::new(UserDataType { device });

        let input_device = unsafe {
            let input_device = lvgl::lv_indev_create();
            lvgl::lv_indev_set_type(input_device, r#type.into());
            lvgl::lv_indev_set_read_cb(input_device, Some(binding_callback_function));
            lvgl::lv_indev_set_user_data(input_device, Box::into_raw(user_data) as *mut c_void);

            if r#type == InputKind::Keypad {
                let group = lvgl::lv_group_get_default();

                lvgl::lv_indev_set_group(input_device, group);
            }

            input_device
        };

        Ok(Self { input_device })
    }
}
