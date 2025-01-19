use Graphics::{Event_code_type, Window_type, LVGL};
use Users::User_identifier_type;

use crate::Error::Result_type;

pub struct Login_type {
    Window: Window_type,
    User_name_text_area: *mut LVGL::lv_obj_t,
    Password_text_area: *mut LVGL::lv_obj_t,
    Button: *mut LVGL::lv_obj_t,
    User: Option<User_identifier_type>,
}

impl Login_type {
    pub fn New() -> Result_type<Self> {
        // - Lock the graphics
        let _Lock = Graphics::Get_instance().Lock();

        // - Create a window
        let Window = Graphics::Get_instance().Create_window()?;

        unsafe {
            LVGL::lv_obj_set_flex_flow(Window.Get_object(), LVGL::LV_FLEX_COLUMN);
            LVGL::lv_obj_set_flex_align(
                Window.Get_object(),
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );
        }

        let User_name_text_area = unsafe {
            // - Create a text area for the user name
            let User_name_text_area = LVGL::lv_textarea_create(Window.Get_object());

            LVGL::lv_textarea_set_placeholder_text(User_name_text_area, c"User name".as_ptr());
            LVGL::lv_textarea_set_one_line(User_name_text_area, true);

            User_name_text_area
        };

        let Password_text_area = unsafe {
            // - Create a text area for the password
            let Password_text_area = LVGL::lv_textarea_create(Window.Get_object());

            LVGL::lv_textarea_set_placeholder_text(Password_text_area, c"Password".as_ptr());
            LVGL::lv_textarea_set_one_line(Password_text_area, true);

            Password_text_area
        };

        let Button = unsafe {
            // - Create a button
            let Button = LVGL::lv_button_create(Window.Get_object());

            let Label = LVGL::lv_label_create(Button);

            LVGL::lv_label_set_text(Label, c"Login".as_ptr());

            Button
        };

        Ok(Login_type {
            Window,
            User_name_text_area,
            Password_text_area,
            Button,
            User: None,
        })
    }

    pub fn Event_handler(&mut self) {
        while let Some(Event) = self.Window.Pop_event() {
            if Event.Get_code() == Event_code_type::Clicked && Event.Get_target() == self.Button {
                self.User = Some(User_identifier_type::Root);
            }
        }
    }

    pub fn Get_logged_user(&self) -> Option<User_identifier_type> {
        self.User
    }
}
