use core::ffi::CStr;

use alloc::string::ToString;
use Graphics::{Event_code_type, Window_type, LVGL};
use Users::User_identifier_type;

use crate::Error::{Error_type, Result_type};

pub struct Login_type {
    window: Window_type,
    user_name_text_area: *mut LVGL::lv_obj_t,
    password_text_area: *mut LVGL::lv_obj_t,
    button: *mut LVGL::lv_obj_t,
    error_label: *mut LVGL::lv_obj_t,
    user: Option<User_identifier_type>,
}

impl Login_type {
    pub async fn new() -> Result_type<Self> {
        // - Lock the graphics
        let _lock = Graphics::get_instance().lock();

        // - Create a window
        let window = Graphics::get_instance().create_window().await?;

        unsafe {
            LVGL::lv_obj_set_flex_flow(window.get_object(), LVGL::LV_FLEX_COLUMN);
            LVGL::lv_obj_set_flex_align(
                window.get_object(),
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );
        }

        let user_name_text_area = unsafe {
            // - Create a text area for the user name
            let user_name_text_area = LVGL::lv_textarea_create(window.get_object());

            LVGL::lv_textarea_set_placeholder_text(user_name_text_area, c"User name".as_ptr());
            LVGL::lv_textarea_set_one_line(user_name_text_area, true);

            user_name_text_area
        };

        let password_text_area = unsafe {
            // - Create a text area for the password
            let password_text_area = LVGL::lv_textarea_create(window.get_object());

            LVGL::lv_textarea_set_placeholder_text(password_text_area, c"Password".as_ptr());
            LVGL::lv_textarea_set_one_line(password_text_area, true);
            LVGL::lv_textarea_set_password_mode(password_text_area, true);

            password_text_area
        };

        let error_label = unsafe {
            // - Create a label for the error
            let error_label = LVGL::lv_label_create(window.get_object());

            LVGL::lv_label_set_text(error_label, c"".as_ptr());

            error_label
        };

        let button = unsafe {
            // - Create a button
            let button = LVGL::lv_button_create(window.get_object());

            let label = LVGL::lv_label_create(button);

            LVGL::lv_label_set_text(label, c"Login".as_ptr());

            button
        };

        Ok(Login_type {
            window,
            user_name_text_area,
            password_text_area,
            button,
            error_label,
            user: None,
        })
    }

    pub fn print_error(&mut self, error: Error_type) {
        let error = error.to_string();
        let error = error.as_bytes();

        unsafe {
            let error = CStr::from_bytes_with_nul_unchecked(error);

            LVGL::lv_label_set_text(self.error_label, error.as_ptr());
        }
    }

    pub fn clear_error(&mut self) {
        unsafe {
            LVGL::lv_label_set_text(self.error_label, c"".as_ptr());
        }
    }

    pub async fn authenticate(&mut self) -> Result_type<()> {
        let (user_name, password) = unsafe {
            let user_name = LVGL::lv_textarea_get_text(self.user_name_text_area);
            let user_name = CStr::from_ptr(user_name);

            let user_name = user_name.to_str().map_err(Error_type::Invalid_UTF_8)?;

            let password = LVGL::lv_textarea_get_text(self.password_text_area);
            let password = CStr::from_ptr(password);
            let password = password.to_str().map_err(Error_type::Invalid_UTF_8)?;

            (user_name, password)
        };

        // - Check the user name and the password
        let user_identifier = Authentication::authenticate_user(
            Virtual_file_system::get_instance(),
            user_name,
            password,
        )
        .await
        .map_err(Error_type::Authentication_failed)?;

        // - Set the user
        let task_manager = Task::get_instance();

        let task = task_manager.get_current_task_identifier().await;

        task_manager
            .set_user(task, user_identifier)
            .await
            .map_err(Error_type::Failed_to_set_task_user)?;

        self.user = Some(user_identifier);

        Ok(())
    }

    pub async fn event_handler(&mut self) {
        while let Some(event) = self.window.pop_event() {
            // If we are typing the user name or the password
            if event.get_code() == Event_code_type::Value_changed
                && (event.get_target() == self.user_name_text_area
                    || event.get_target() == self.password_text_area)
            {
                self.clear_error();
            }
            // If the "Login" button is clicked
            else if event.get_code() == Event_code_type::Clicked
                && event.get_target() == self.button
            {
                let result = self.authenticate().await;

                if let Err(error) = result {
                    self.print_error(error);
                }
            }
        }
    }

    pub fn get_logged_user(&self) -> Option<User_identifier_type> {
        self.user
    }
}
