use alloc::{format, string::String};
use graphics::lvgl;
use shared::unix_to_human_time;

use crate::error::{Error_type, Result_type};

pub struct Layout_type {
    window: *mut lvgl::lv_obj_t,
    _header: *mut lvgl::lv_obj_t,
    _body: *mut lvgl::lv_obj_t,
    clock: *mut lvgl::lv_obj_t,
    clock_string: String,
    _battery: *mut lvgl::lv_obj_t,
    _wi_fi: *mut lvgl::lv_obj_t,
}

impl Drop for Layout_type {
    fn drop(&mut self) {
        unsafe { lvgl::lv_obj_delete(self.window) }
    }
}

impl Layout_type {
    pub async fn r#loop(&mut self) {
        self.update_clock().await;
    }

    async fn update_clock(&mut self) {
        // - Update the clock
        let current_time = time::get_instance().get_current_time();

        if let Ok(current_time) = current_time {
            let (_, _, _, hour, minute, _) = unix_to_human_time(current_time.as_seconds() as i64);

            let _ = graphics::get_instance().lock().await;

            self.clock_string = format!("{hour:02}:{minute:02}\0");

            unsafe {
                lvgl::lv_label_set_text_static(self.clock, self.clock_string.as_ptr() as *const i8);
            }
        }
    }

    pub fn get_windows_parent(&self) -> *mut lvgl::lv_obj_t {
        self.window
    }

    pub async fn new() -> Result_type<Self> {
        let _lock = graphics::get_instance().lock().await; // Lock the graphics

        // - Create a window
        let window = unsafe {
            let window = lvgl::lv_screen_active();

            if window.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_set_size(window, lvgl::lv_pct(100), lvgl::lv_pct(100));
            lvgl::lv_obj_set_flex_flow(window, lvgl::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            lvgl::lv_obj_set_style_pad_all(window, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_pad_row(window, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_border_width(window, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_radius(window, 0, lvgl::LV_STATE_DEFAULT);

            window
        };

        // - Create a header
        let header = unsafe {
            let header = lvgl::lv_obj_create(window);

            if header.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }
            lvgl::lv_obj_set_size(header, lvgl::lv_pct(100), 32);
            lvgl::lv_obj_set_style_border_width(header, 0, lvgl::LV_STATE_DEFAULT); // Remove the default border
            lvgl::lv_obj_set_style_pad_all(header, 8, lvgl::LV_STATE_DEFAULT); // Remove the default padding
            lvgl::lv_obj_set_style_bg_color(header, lvgl::lv_color_black(), lvgl::LV_STATE_DEFAULT);
            // Set the background color to black
            lvgl::lv_obj_set_style_text_color(
                header,
                lvgl::lv_color_white(),
                lvgl::LV_STATE_DEFAULT,
            ); // Set the text color to white for children
            lvgl::lv_obj_set_style_radius(header, 0, lvgl::LV_STATE_DEFAULT); // Remove the default radius

            header
        };

        // - - Create a label for the clock
        let clock = unsafe {
            let clock = lvgl::lv_label_create(header);

            if clock.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_set_align(clock, lvgl::lv_align_t_LV_ALIGN_CENTER);

            clock
        };

        // - - Create a label for the battery
        let battery = unsafe {
            // - - Create a label for the battery
            let battery = lvgl::lv_label_create(header);

            if battery.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_set_align(battery, lvgl::lv_align_t_LV_ALIGN_RIGHT_MID);
            lvgl::lv_label_set_text(battery, lvgl::LV_SYMBOL_BATTERY_3 as *const u8 as *const i8);

            battery
        };

        // - - Create a label for the WiFi

        let wi_fi = unsafe {
            // - - Create a label for the WiFi

            let wi_fi = lvgl::lv_label_create(header);

            if wi_fi.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_align_to(wi_fi, battery, lvgl::lv_align_t_LV_ALIGN_OUT_LEFT_MID, 0, 0);
            lvgl::lv_label_set_text(wi_fi, lvgl::LV_SYMBOL_WIFI as *const u8 as *const i8);

            wi_fi
        };

        // - - Create a body object
        let body = unsafe {
            // - Create a body
            let body = lvgl::lv_obj_create(window);

            if body.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_set_width(body, lvgl::lv_pct(100));
            lvgl::lv_obj_set_flex_grow(body, 1);
            lvgl::lv_obj_set_style_pad_all(body, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_border_width(body, 0, lvgl::LV_STATE_DEFAULT);

            body
        };

        drop(_lock); // Unlock the graphics

        graphics::get_instance().set_window_parent(body).await?;

        Ok(Self {
            window,
            _header: header,
            _body: body,
            clock,
            clock_string: String::with_capacity(6),
            _battery: battery,
            _wi_fi: wi_fi,
        })
    }
}
