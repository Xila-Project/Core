pub(crate) use core::ffi::CStr;

use alloc::string::ToString;
use xila::authentication;
use xila::graphics::{self, EventKind, Window, lvgl};
use xila::task;
use xila::users::UserIdentifier;
use xila::virtual_file_system;

use crate::error::{Error, Result};
use crate::translations;

pub struct Login {
    window: Window,
    user_name_text_area: *mut lvgl::lv_obj_t,
    password_text_area: *mut lvgl::lv_obj_t,
    button: *mut lvgl::lv_obj_t,
    error_label: *mut lvgl::lv_obj_t,
    user: Option<UserIdentifier>,
}

impl Login {
    pub async fn new() -> Result<Self> {
        // - Lock the graphics
        let _lock = graphics::get_instance().lock();

        // - Create a window
        let window = graphics::get_instance().create_window().await?;

        unsafe {
            lvgl::lv_obj_set_flex_flow(window.get_object(), lvgl::LV_FLEX_COLUMN);
            lvgl::lv_obj_set_flex_align(
                window.get_object(),
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );
        }

        let user_name_text_area = unsafe {
            // - Create a text area for the user name
            let user_name_text_area = lvgl::lv_textarea_create(window.get_object());

            lvgl::lv_textarea_set_placeholder_text(
                user_name_text_area,
                translations::login__user_name!(c).as_ptr(),
            );
            lvgl::lv_textarea_set_one_line(user_name_text_area, true);

            user_name_text_area
        };

        let password_text_area = unsafe {
            // - Create a text area for the password
            let password_text_area = lvgl::lv_textarea_create(window.get_object());

            lvgl::lv_textarea_set_placeholder_text(
                password_text_area,
                translations::login__password!(c).as_ptr(),
            );
            lvgl::lv_textarea_set_one_line(password_text_area, true);
            lvgl::lv_textarea_set_password_mode(password_text_area, true);

            password_text_area
        };

        let error_label = unsafe {
            // - Create a label for the error
            let error_label = lvgl::lv_label_create(window.get_object());

            lvgl::lv_label_set_text(error_label, c"".as_ptr());

            error_label
        };

        let button = unsafe {
            // - Create a button
            let button = lvgl::lv_button_create(window.get_object());

            let label = lvgl::lv_label_create(button);

            lvgl::lv_label_set_text(label, translations::login__login!(c).as_ptr());

            button
        };

        Ok(Login {
            window,
            user_name_text_area,
            password_text_area,
            button,
            error_label,
            user: None,
        })
    }

    pub fn print_error(&mut self, error: Error) {
        let error = error.to_string();
        let error = error.as_bytes();

        unsafe {
            let error = CStr::from_bytes_with_nul_unchecked(error);

            lvgl::lv_label_set_text(self.error_label, error.as_ptr());
        }
    }

    pub fn clear_error(&mut self) {
        unsafe {
            lvgl::lv_label_set_text(self.error_label, c"".as_ptr());
        }
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        let (user_name, password) = unsafe {
            let user_name = lvgl::lv_textarea_get_text(self.user_name_text_area);
            let user_name = CStr::from_ptr(user_name);

            let user_name = user_name.to_str().map_err(Error::InvalidUtf8)?;

            let password = lvgl::lv_textarea_get_text(self.password_text_area);
            let password = CStr::from_ptr(password);
            let password = password.to_str().map_err(Error::InvalidUtf8)?;

            (user_name, password)
        };

        // - Check the user name and the password
        let user_identifier = authentication::authenticate_user(
            virtual_file_system::get_instance(),
            user_name,
            password,
        )
        .await
        .map_err(Error::AuthenticationFailed)?;

        // - Set the user
        let task_manager = task::get_instance();

        let task = task_manager.get_current_task_identifier().await;

        task_manager
            .set_user(task, user_identifier)
            .await
            .map_err(Error::FailedToSetTaskUser)?;

        self.user = Some(user_identifier);

        Ok(())
    }

    pub async fn event_handler(&mut self) {
        while let Some(event) = self.window.pop_event() {
            // If we are typing the user name or the password
            if event.get_code() == EventKind::ValueChanged
                && (event.get_target() == self.user_name_text_area
                    || event.get_target() == self.password_text_area)
            {
                self.clear_error();
            }
            // If the "Login" button is clicked
            else if event.get_code() == EventKind::Clicked && event.get_target() == self.button {
                let result = self.authenticate().await;

                if let Err(error) = result {
                    self.print_error(error);
                }
            }
        }
    }

    pub fn get_logged_user(&self) -> Option<UserIdentifier> {
        self.user
    }
}
