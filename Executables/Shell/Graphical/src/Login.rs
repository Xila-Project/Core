use core::ffi::CStr;

use alloc::string::ToString;
use Graphics::{Event_code_type, Window_type, LVGL};
use Users::User_identifier_type;

use crate::Error::{Error_type, Result_type};

pub struct Login_type {
    Window: Window_type,
    User_name_text_area: *mut LVGL::lv_obj_t,
    Password_text_area: *mut LVGL::lv_obj_t,
    Button: *mut LVGL::lv_obj_t,
    Error_label: *mut LVGL::lv_obj_t,
    User: Option<User_identifier_type>,
}

impl Login_type {
    pub async fn New() -> Result_type<Self> {
        // - Lock the graphics
        let _Lock = Graphics::Get_instance().Lock();

        // - Create a window
        let Window = Graphics::Get_instance().Create_window().await?;

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
            LVGL::lv_textarea_set_password_mode(Password_text_area, true);

            Password_text_area
        };

        let Error_label = unsafe {
            // - Create a label for the error
            let Error_label = LVGL::lv_label_create(Window.Get_object());

            LVGL::lv_label_set_text(Error_label, c"".as_ptr());

            Error_label
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
            Error_label,
            User: None,
        })
    }

    pub fn Print_error(&mut self, Error: Error_type) {
        let Error = Error.to_string();
        let Error = Error.as_bytes();

        unsafe {
            let Error = CStr::from_bytes_with_nul_unchecked(Error);

            LVGL::lv_label_set_text(self.Error_label, Error.as_ptr());
        }
    }

    pub fn Clear_error(&mut self) {
        unsafe {
            LVGL::lv_label_set_text(self.Error_label, c"".as_ptr());
        }
    }

    pub async fn Authenticate(&mut self) -> Result_type<()> {
        let (User_name, Password) = unsafe {
            let User_name = LVGL::lv_textarea_get_text(self.User_name_text_area);
            let User_name = CStr::from_ptr(User_name);

            let User_name = User_name.to_str().map_err(Error_type::Invalid_UTF_8)?;

            let Password = LVGL::lv_textarea_get_text(self.Password_text_area);
            let Password = CStr::from_ptr(Password);
            let Password = Password.to_str().map_err(Error_type::Invalid_UTF_8)?;

            (User_name, Password)
        };

        // - Check the user name and the password
        let User_identifier = Authentication::Authenticate_user(
            Virtual_file_system::Get_instance(),
            User_name,
            Password,
        )
        .await
        .map_err(Error_type::Authentication_failed)?;

        // - Set the user
        let Task_manager = Task::Get_instance();

        let Task = Task_manager.Get_current_task_identifier().await;

        Task_manager
            .Set_user(Task, User_identifier)
            .await
            .map_err(Error_type::Failed_to_set_task_user)?;

        self.User = Some(User_identifier);

        Ok(())
    }

    pub async fn Event_handler(&mut self) {
        while let Some(Event) = self.Window.Pop_event() {
            // If we are typing the user name or the password
            if Event.Get_code() == Event_code_type::Value_changed
                && (Event.Get_target() == self.User_name_text_area
                    || Event.Get_target() == self.Password_text_area)
            {
                self.Clear_error();
            }
            // If the "Login" button is clicked
            else if Event.Get_code() == Event_code_type::Clicked
                && Event.Get_target() == self.Button
            {
                let Result = self.Authenticate().await;

                if let Err(Error) = Result {
                    self.Print_error(Error);
                }
            }
        }
    }

    pub fn Get_logged_user(&self) -> Option<User_identifier_type> {
        self.User
    }
}
