use alloc::boxed::Box;
use file_system::{ControlArgument, DirectCharacterDevice};

use core::{ffi::c_void, ptr::null_mut, slice};

use crate::{
    Area, GET_RESOLUTION, Point, RenderingColor, Result, SET_DRAWING_AREA, draw_buffer::Buffer,
};

use super::lvgl;

struct UserData {
    device: &'static dyn DirectCharacterDevice,
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
    let mut area: Area = unsafe { *area }.into();

    let buffer_size: usize =
        (area.get_width()) as usize * (area.get_height()) as usize * size_of::<RenderingColor>();

    let buffer = unsafe { slice::from_raw_parts(data as *const u8, buffer_size) };

    let user_data = unsafe { &mut *(lvgl::lv_display_get_user_data(display) as *mut UserData) };

    let device = &user_data.device;

    device
        .control(SET_DRAWING_AREA, ControlArgument::from(&mut area))
        .expect("Error setting drawing area");

    device.write(buffer, 0).expect("Error writing to display");

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
        device: &'static dyn DirectCharacterDevice,
        buffer_size: usize,
        double_buffered: bool,
    ) -> Result<Self> {
        // Get the resolution from the device.
        let mut resolution = Point::new(0, 0);
        device
            .control(GET_RESOLUTION, ControlArgument::from(&mut resolution))
            .expect("Error getting resolution");

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
        let user_data = Box::new(UserData { device });

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

    pub fn check_for_resizing(&self) {
        let user_data =
            unsafe { &mut *(lvgl::lv_display_get_user_data(self.display) as *mut UserData) };

        let mut was_resize = false;

        user_data
            .device
            .control(
                crate::screen::WAS_RESIZED,
                ControlArgument::from(&mut was_resize),
            )
            .expect("Error checking if display was resized");

        if !was_resize {
            return;
        }

        let mut resolution = Point::new(0, 0);

        user_data
            .device
            .control(GET_RESOLUTION, ControlArgument::from(&mut resolution))
            .expect("Error getting resolution");

        unsafe {
            lvgl::lv_display_set_resolution(
                self.display,
                resolution.get_x() as i32,
                resolution.get_y() as i32,
            );
        }
    }

    pub fn get_lvgl_display(&self) -> *mut lvgl::lv_display_t {
        self.display
    }

    pub fn get_object(&self) -> *mut lvgl::lv_obj_t {
        unsafe { lvgl::lv_display_get_screen_active(self.display) }
    }
}
