use Graphics::{
    Window_type,
    LVGL::{self, lv_obj_create},
};
use Shared::Unix_to_human_time;

use crate::Error::{Error_type, Result_type};

pub struct Layout_type {
    pub _Window: *mut LVGL::lv_obj_t,
    _Header: *mut LVGL::lv_obj_t,
    _Body: *mut LVGL::lv_obj_t,
    Clock: *mut LVGL::lv_obj_t,
    Clock_string: String,
    _Battery: *mut LVGL::lv_obj_t,
    _WiFi: *mut LVGL::lv_obj_t,
}

impl Layout_type {
    pub fn Loop(&mut self) {
        self.Update_clock();
    }

    fn Update_clock(&mut self) {
        // - Update the clock
        let Current_time = Time::Get_instance().Get_current_time();

        if let Ok(Current_time) = Current_time {
            let (_, _, _, Hour, Minute, _) = Unix_to_human_time(Current_time.As_seconds() as i64);

            if let Ok(_Lock) = Graphics::Get_instance().Lock() {
                self.Clock_string = format!("{:02}:{:02}\0", Hour, Minute);

                unsafe {
                    LVGL::lv_label_set_text_static(
                        self.Clock,
                        self.Clock_string.as_ptr() as *const i8,
                    );
                }
            }
        }
    }

    pub fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock()?; // Lock the graphics

        // - Create a window
        let Window = unsafe {
            let Window = LVGL::lv_screen_active();

            if Window.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(Window, LVGL::lv_pct(100), LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_flow(Window, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            LVGL::lv_obj_set_style_pad_all(Window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_row(Window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(Window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(Window, 0, LVGL::LV_STATE_DEFAULT);

            Window
        };

        // - Create a header
        let Header = unsafe {
            let Header = LVGL::lv_obj_create(Window);

            if Header.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }
            LVGL::lv_obj_set_size(Header, LVGL::lv_pct(100), 32);
            LVGL::lv_obj_set_style_border_width(Header, 0, LVGL::LV_STATE_DEFAULT); // Remove the default border
            LVGL::lv_obj_set_style_pad_all(Header, 8, LVGL::LV_STATE_DEFAULT); // Remove the default padding
            LVGL::lv_obj_set_style_bg_color(Header, LVGL::lv_color_black(), LVGL::LV_STATE_DEFAULT);
            // Set the background color to black
            LVGL::lv_obj_set_style_text_color(
                Header,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            ); // Set the text color to white for children
            LVGL::lv_obj_set_style_radius(Header, 0, LVGL::LV_STATE_DEFAULT); // Remove the default radius

            Header
        };

        // - - Create a label for the clock
        let Clock = unsafe {
            let Clock = LVGL::lv_label_create(Header);

            if Clock.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_align(Clock, LVGL::lv_align_t_LV_ALIGN_CENTER);

            Clock
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

        Graphics::Get_instance().Set_window_parent(Body)?;

        Ok(Self {
            _Window: Window,
            _Header: Header,
            _Body: Body,
            Clock,
            Clock_string: String::with_capacity(6),
            _Battery: Battery,
            _WiFi: WiFi,
        })
    }
}
