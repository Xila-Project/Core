use core::slice;
use std::{pin::Pin, ptr::null_mut};

use File_system::File_type;

use crate::{
    Area_type, Color_type, Draw_buffer::Buffer_type, Point_type, Result_type,
    Screen_write_data_type,
};

use super::lvgl;

pub struct Display_type {
    Display: lvgl::lv_display_t,
    _Buffer_1: Buffer,
    _Buffer_2: Option<Pin<Box<[MaybeUninit<lvgl::lv_color_t>; Buffer_size]>>>,
}

unsafe impl Send for Display_type {}

unsafe impl Sync for Display_type {}

pub extern "C" fn Binding_callback_function(
    Display: *mut lvgl::lv_disp_t,
    Area: *const lvgl::lv_area_t,
    Data: *mut u8,
) {
    let Area: Area_type = unsafe { *Area }.into();

    let Buffer_size: usize = Area.Get_width() as usize * Area.Get_height() as usize;

    let Buffer = unsafe { slice::from_raw_parts_mut(Data as *mut Color_type, Buffer_size) };

    let Screen_write_data = Screen_write_data_type::New(Area, Buffer);

    File.Write(Screen_write_data.as_ref())
        .expect("Error writing to display");

    lvgl::lv_display_flush_ready(Display);
}

impl Drop for Display_type {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_display_delete(self.Display);
        }
    }
}

impl Display_type {
    pub fn New<const Buffer_size: usize>(
        File: File_type,
        Resolution: Point_type,
        Double_buffered: bool,
    ) -> Result_type<Self> {
        // Create the display.
        let LVGL_display = unsafe {
            lvgl::lv_display_create(Resolution.Get_x() as i32, Resolution.Get_y() as i32)
        };

        let Buffer_1: *mut Color_type = Buffer_type::<Buffer_size>::default().into();

        let Buffer_2: *mut lv_color_t = if Double_buffered {
            Buffer_type::<Buffer_size>::default().into()
        } else {
            null_mut()
        };

        // Set the buffer(s) and the render mode.
        lvgl::lv_display_set_buffer(
            LVGL_display,
            Buffer_1,
            Buffer_2,
            Buffer_size,
            lvgl::lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
        );

        // Set the flush callback.
        lvgl::lv_display_set_flush_cb(LVGL_display, Some(Binding_callback_function));

        Ok(Self {
            Display: LVGL_display,
            _Buffer_1: Pin::from(unsafe { Box::from_raw(Buffer_1) }),
            _Buffer_2: if Double_buffered {
                Some(Pin::from(unsafe { Box::from_raw(Buffer_2) }))
            } else {
                None
            },
        })
    }

    pub fn Get_object(&self) -> Result_type<lvgl::Screen> {
        Ok(self.Display.get_scr_act()?)
    }
}
