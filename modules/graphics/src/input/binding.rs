use file_system::Device;

use crate::lvgl;

use super::Input_data_type;

/// This function is called by LVGL when it needs to read input data.
///
/// # Safety
///
/// This function may dereference a raw pointer.
pub unsafe extern "C" fn binding_callback_function(
    input_device: *mut lvgl::lv_indev_t,
    data: *mut lvgl::lv_indev_data_t,
) {
    let user_data = unsafe { lvgl::lv_indev_get_user_data(input_device) as *mut UserData };

    let device = unsafe { &(*user_data).device };

    let mut pointer_data = Input_data_type::default();

    device
        .read(pointer_data.as_mut())
        .expect("Error reading from input device");

    unsafe {
        *data = pointer_data.into();
    }
}

pub struct UserData {
    pub device: Device,
}
