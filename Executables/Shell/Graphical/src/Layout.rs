use alloc::{format, string::String};
use Graphics::LVGL;
use Shared::Unix_to_human_time;

use crate::Error::{Error_type, Result_type};

pub struct Layout_type {
    window: *mut LVGL::lv_obj_t,
    _header: *mut LVGL::lv_obj_t,
    _body: *mut LVGL::lv_obj_t,
    clock: *mut LVGL::lv_obj_t,
    clock_string: String,
    _battery: *mut LVGL::lv_obj_t,
    _wi_fi: *mut LVGL::lv_obj_t,
}

impl Drop for Layout_type {
    fn drop(&mut self) {
        unsafe { LVGL::lv_obj_delete(self.window) }
    }
}

impl Layout_type {
    pub async fn Loop(&mut self) {
        self.Update_clock().await;
    }

    async fn Update_clock(&mut self) {
        // - Update the clock
        let Current_time = Time::Get_instance().Get_current_time();

        if let Ok(Current_time) = Current_time {
            let (_, _, _, hour, minute, _) = Unix_to_human_time(Current_time.As_seconds() as i64);

            let _ = Graphics::Get_instance().Lock().await;

            self.clock_string = format!("{hour:02}:{minute:02}\0");

            unsafe {
                LVGL::lv_label_set_text_static(self.clock, self.clock_string.as_ptr() as *const i8);
            }
        }
    }

    pub fn Get_windows_parent(&self) -> *mut LVGL::lv_obj_t {
        self.window
    }

    pub async fn New() -> Result_type<Self> {
        let _lock = Graphics::Get_instance().Lock().await; // Lock the graphics

        // - Create a window
        let Window = unsafe {
            let window = LVGL::lv_screen_active();

            if window.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(window, LVGL::lv_pct(100), LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_flow(window, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            LVGL::lv_obj_set_style_pad_all(window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_row(window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(window, 0, LVGL::LV_STATE_DEFAULT);

            window
        };

        // - Create a header
        let Header = unsafe {
            let header = LVGL::lv_obj_create(Window);

            if header.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }
            LVGL::lv_obj_set_size(header, LVGL::lv_pct(100), 32);
            LVGL::lv_obj_set_style_border_width(header, 0, LVGL::LV_STATE_DEFAULT); // Remove the default border
            LVGL::lv_obj_set_style_pad_all(header, 8, LVGL::LV_STATE_DEFAULT); // Remove the default padding
            LVGL::lv_obj_set_style_bg_color(header, LVGL::lv_color_black(), LVGL::LV_STATE_DEFAULT);
            // Set the background color to black
            LVGL::lv_obj_set_style_text_color(
                header,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            ); // Set the text color to white for children
            LVGL::lv_obj_set_style_radius(header, 0, LVGL::LV_STATE_DEFAULT); // Remove the default radius

            header
        };

        // - - Create a label for the clock
        let Clock = unsafe {
            let clock = LVGL::lv_label_create(Header);

            if clock.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_align(clock, LVGL::lv_align_t_LV_ALIGN_CENTER);

            clock
        };

        // - - Create a label for the battery
        let Battery = unsafe {
            // - - Create a label for the battery
            let Battery = LVGL::lv_label_create(Header);

            if Battery.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_align(Battery, LVGL::lv_align_t_LV_ALIGN_RIGHT_MID);
            LVGL::lv_label_set_text(Battery, LVGL::LV_SYMBOL_BATTERY_3 as *const u8 as *const i8);

            Battery
        };

        // - - Create a label for the WiFi

        let WiFi = unsafe {
            // - - Create a label for the WiFi

            let WiFi = LVGL::lv_label_create(Header);

            if WiFi.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_align_to(WiFi, Battery, LVGL::lv_align_t_LV_ALIGN_OUT_LEFT_MID, 0, 0);
            LVGL::lv_label_set_text(WiFi, LVGL::LV_SYMBOL_WIFI as *const u8 as *const i8);

            WiFi
        };

        // - - Create a body object
        let Body = unsafe {
            // - Create a body
            let Body = LVGL::lv_obj_create(Window);

            if Body.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_width(Body, LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_grow(Body, 1);
            LVGL::lv_obj_set_style_pad_all(Body, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(Body, 0, LVGL::LV_STATE_DEFAULT);

            Body
        };

        drop(_lock); // Unlock the graphics

        Graphics::Get_instance().Set_window_parent(Body).await?;

        Ok(Self {
            window: Window,
            _header: Header,
            _body: Body,
            clock: Clock,
            clock_string: String::with_capacity(6),
            _battery: Battery,
            _wi_fi: WiFi,
        })
    }
}
