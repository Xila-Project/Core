use File_system::Device_type;

use crate::LVGL;

use super::Input_data_type;

/// This function is called by LVGL when it needs to read input data.
///
/// # Safety
///
/// This function may dereference a raw pointer.
pub unsafe extern "C" fn Binding_callback_function(
    Input_device: *mut LVGL::lv_indev_t,
    Data: *mut LVGL::lv_indev_data_t,
) {
    let User_data = unsafe { LVGL::lv_indev_get_user_data(Input_device) as *mut User_data_type };

    let Device = &(*User_data).Device;

    let mut Pointer_data = Input_data_type::default();

    Device
        .Read(Pointer_data.as_mut())
        .expect("Error reading from input device");

    unsafe {
        *Data = Pointer_data.into();
    }
}

pub struct User_data_type {
    pub Device: Device_type,
}
