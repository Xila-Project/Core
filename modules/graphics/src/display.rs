use alloc::boxed::Box;

use core::{ffi::c_void, ptr::null_mut, slice};

use file_system::Device;

use crate::{Area, Point, RenderingColor, Result, ScreenWriteData, draw_buffer::Buffer};

use super::lvgl;

struct UserData {
    device: Device,
}

pub struct Display {
    display: *mut lvgl::lv_display_t,
    _buffer_1: Buffer,
    _buffer_2: Option<Buffer>,
}

unsafe impl Send for Display {}

unsafe impl Sync for Display {}

unsafe extern "C" fn binding_callback_function(
    display: *mut lvgl::lv_disp_t,
    area: *const lvgl::lv_area_t,
    data: *mut u8,
) {
    let area: Area = unsafe { *area }.into();

    let buffer_size: usize = (area.get_width()) as usize * (area.get_height()) as usize;

    let buffer = unsafe { slice::from_raw_parts_mut(data as *mut RenderingColor, buffer_size) };

    let screen_write_data = ScreenWriteData::new(area, buffer);

    let user_data = unsafe { &*(lvgl::lv_display_get_user_data(display) as *mut UserData) };

    let device = &user_data.device;

    device
        .write(screen_write_data.as_ref())
        .expect("Error writing to display");

    unsafe { lvgl::lv_display_flush_ready(display) };
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_display_delete(self.display);
        }
    }
}

impl Display {
    pub fn new(
        file: Device,
        resolution: Point,
        buffer_size: usize,
        double_buffered: bool,
    ) -> Result<Self> {
        // Create the display.
        let lvgl_display: *mut lvgl_rust_sys::_lv_display_t = unsafe {
            lvgl::lv_display_create(resolution.get_x() as i32, resolution.get_y() as i32)
        };

        // Set the buffer(s) and the render mode.
        let buffer_1 = Buffer::new(buffer_size);

        let buffer_2 = if double_buffered {
            Some(Buffer::new(buffer_size))
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
        let user_data = Box::new(UserData { device: file });

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

    pub fn get_lvgl_display(&self) -> *mut lvgl::lv_display_t {
        self.display
    }

    pub fn get_object(&self) -> *mut lvgl::lv_obj_t {
        unsafe { lvgl::lv_display_get_screen_active(self.display) }
    }
}
