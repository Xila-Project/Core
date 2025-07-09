use alloc::boxed::Box;

use core::{ffi::c_void, ptr::null_mut, slice};

use file_system::Device_type;

use crate::{
    draw_buffer::Buffer_type, Area_type, Point_type, Rendering_color_type, Result_type,
    Screen_write_data_type,
};

use super::lvgl;

struct User_data {
    device: Device_type,
}

pub struct Display_type {
    display: *mut lvgl::lv_display_t,
    _buffer_1: Buffer_type,
    _buffer_2: Option<Buffer_type>,
}

unsafe impl Send for Display_type {}

unsafe impl Sync for Display_type {}

unsafe extern "C" fn binding_callback_function(
    display: *mut lvgl::lv_disp_t,
    area: *const lvgl::lv_area_t,
    data: *mut u8,
) {
    let area: Area_type = unsafe { *area }.into();

    let buffer_size: usize = (area.get_width()) as usize * (area.get_height()) as usize;

    let buffer =
        unsafe { slice::from_raw_parts_mut(data as *mut Rendering_color_type, buffer_size) };

    let screen_write_data = Screen_write_data_type::new(area, buffer);

    let user_data = unsafe { &*(lvgl::lv_display_get_user_data(display) as *mut User_data) };

    let device = &user_data.device;

    device
        .Write(screen_write_data.as_ref())
        .expect("Error writing to display");

    lvgl::lv_display_flush_ready(display);
}

impl Drop for Display_type {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_display_delete(self.display);
        }
    }
}

impl Display_type {
    pub fn new(
        file: Device_type,
        resolution: Point_type,
        buffer_size: usize,
        double_buffered: bool,
    ) -> Result_type<Self> {
        // Create the display.
        let lvgl_display: *mut lvgl_rust_sys::_lv_display_t = unsafe {
            lvgl::lv_display_create(resolution.get_x() as i32, resolution.get_y() as i32)
        };

        // Set the buffer(s) and the render mode.
        let buffer_1 = Buffer_type::New(buffer_size);

        let buffer_2 = if double_buffered {
            Some(Buffer_type::New(buffer_size))
        } else {
            None
        };

        unsafe {
            lvgl::lv_display_set_buffers(
                lvgl_display,
                buffer_1.as_ref().as_ptr() as *mut c_void,
                buffer_2
                    .as_ref()
                    .map_or(null_mut(), |buffer| buffer.as_ref().as_ptr() as *mut c_void),
                buffer_size as u32,
                lvgl::lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
            )
        }

        // Set the user data.
        let user_data = Box::new(User_data { device: file });

        unsafe {
            lvgl::lv_display_set_user_data(lvgl_display, Box::into_raw(user_data) as *mut c_void)
        };

        // Set the flush callback.
        unsafe { lvgl::lv_display_set_flush_cb(lvgl_display, Some(binding_callback_function)) }

        Ok(Self {
            display: lvgl_display,
            _buffer_1: buffer_1,
            _buffer_2: buffer_2,
        })
    }

    pub fn get_object(&self) -> *mut lvgl::lv_obj_t {
        unsafe { lvgl::lv_display_get_screen_active(self.display) }
    }
}
