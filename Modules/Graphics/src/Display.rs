use core::slice;
use std::{ffi::c_void, ptr::null_mut};

use File_system::Device_type;

use crate::{
    Area_type, Color_type, Draw_buffer::Buffer_type, Point_type, Result_type,
    Screen_write_data_type,
};

use super::LVGL;

struct User_data {
    Device: Device_type,
}

pub struct Display_type {
    Display: *mut LVGL::lv_display_t,
    _Buffer_1: Buffer_type,
    _Buffer_2: Option<Buffer_type>,
}

unsafe impl Send for Display_type {}

unsafe impl Sync for Display_type {}

unsafe extern "C" fn Binding_callback_function(
    Display: *mut LVGL::lv_disp_t,
    Area: *const LVGL::lv_area_t,
    Data: *mut u8,
) {
    let Area: Area_type = unsafe { *Area }.into();

    let Buffer_size: usize = (Area.Get_width()) as usize * (Area.Get_height()) as usize;

    let Buffer = unsafe { slice::from_raw_parts_mut(Data as *mut Color_type, Buffer_size) };

    let Screen_write_data = Screen_write_data_type::New(Area, Buffer);

    let User_data = unsafe { &*(LVGL::lv_display_get_user_data(Display) as *mut User_data) };

    let Device = &User_data.Device;

    Device
        .Write(Screen_write_data.as_ref())
        .expect("Error writing to display");

    LVGL::lv_display_flush_ready(Display);
}

impl Drop for Display_type {
    fn drop(&mut self) {
        unsafe {
            LVGL::lv_display_delete(self.Display);
        }
    }
}

impl Display_type {
    pub fn New(
        File: Device_type,
        Resolution: Point_type,
        Buffer_size: usize,
        Double_buffered: bool,
    ) -> Result_type<Self> {
        // Create the display.
        let LVGL_display = unsafe {
            LVGL::lv_display_create(Resolution.Get_x() as i32, Resolution.Get_y() as i32)
        };

        // Set the buffer(s) and the render mode.
        let Buffer_1 = Buffer_type::New(Buffer_size);

        let Buffer_2 = if Double_buffered {
            Some(Buffer_type::New(Buffer_size))
        } else {
            None
        };

        unsafe {
            LVGL::lv_display_set_buffers(
                LVGL_display,
                Buffer_1.as_ref().as_ptr() as *mut c_void,
                Buffer_2
                    .as_ref()
                    .map_or(null_mut(), |Buffer| Buffer.as_ref().as_ptr() as *mut c_void),
                Buffer_size as u32,
                LVGL::lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
            )
        }

        // Set the user data.
        let User_data = Box::new(User_data { Device: File });

        unsafe {
            LVGL::lv_display_set_user_data(LVGL_display, Box::into_raw(User_data) as *mut c_void)
        };

        // Set the flush callback.
        unsafe { LVGL::lv_display_set_flush_cb(LVGL_display, Some(Binding_callback_function)) }

        Ok(Self {
            Display: LVGL_display,
            _Buffer_1: Buffer_1,
            _Buffer_2: Buffer_2,
        })
    }

    pub fn Get_object(&self) -> *mut LVGL::lv_obj_t {
        unsafe { LVGL::lv_display_get_screen_active(self.Display) }
    }
}
