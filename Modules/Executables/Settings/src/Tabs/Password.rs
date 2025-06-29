use crate::Error::Result_type;
use alloc::ffi::CString;
use Authentication;
use Graphics::{Event_code_type, Event_type, LVGL};
use Task;
use Users;
use Virtual_file_system;

pub struct Password_tab_type {
    Tab_container: *mut LVGL::lv_obj_t,
    Current_password_text_area: *mut LVGL::lv_obj_t,
    New_password_text_area: *mut LVGL::lv_obj_t,
    Confirm_password_text_area: *mut LVGL::lv_obj_t,
    Change_password_button: *mut LVGL::lv_obj_t,
    Password_status_label: *mut LVGL::lv_obj_t,
}

impl Password_tab_type {
    pub fn New() -> Self {
        Self {
            Tab_container: core::ptr::null_mut(),
            Current_password_text_area: core::ptr::null_mut(),
            New_password_text_area: core::ptr::null_mut(),
            Confirm_password_text_area: core::ptr::null_mut(),
            Change_password_button: core::ptr::null_mut(),
            Password_status_label: core::ptr::null_mut(),
        }
    }

    async fn Handle_password_change(&mut self) {
        // Get the current user
        let task_manager = Task::Get_instance();
        let current_task = task_manager.Get_current_task_identifier().await;

        let user_id = match task_manager.Get_user(current_task).await {
            Ok(user_id) => user_id,
            Err(_) => {
                self.Set_password_status("Failed to get current user", true)
                    .await;
                return;
            }
        };

        let users_manager = Users::Get_instance();
        let username = match users_manager.Get_user_name(user_id).await {
            Ok(name) => name,
            Err(_) => {
                self.Set_password_status("Failed to get username", true)
                    .await;
                return;
            }
        };

        // Get passwords from text areas
        let (current_password, new_password, confirm_password) = unsafe {
            let current = core::ffi::CStr::from_ptr(LVGL::lv_textarea_get_text(
                self.Current_password_text_area,
            ));
            let New =
                core::ffi::CStr::from_ptr(LVGL::lv_textarea_get_text(self.New_password_text_area));
            let confirm = core::ffi::CStr::from_ptr(LVGL::lv_textarea_get_text(
                self.Confirm_password_text_area,
            ));

            let current = match current.to_str() {
                Ok(s) => s,
                Err(_) => {
                    self.Set_password_status("Invalid characters in current password", true)
                        .await;
                    return;
                }
            };

            let New = match New.to_str() {
                Ok(s) => s,
                Err(_) => {
                    self.Set_password_status("Invalid characters in new password", true)
                        .await;
                    return;
                }
            };

            let confirm = match confirm.to_str() {
                Ok(s) => s,
                Err(_) => {
                    self.Set_password_status("Invalid characters in confirm password", true)
                        .await;
                    return;
                }
            };

            (current, New, confirm)
        };

        // Validate inputs
        if current_password.is_empty() {
            self.Set_password_status("Current password cannot be empty", true)
                .await;
            return;
        }

        if new_password.is_empty() {
            self.Set_password_status("New password cannot be empty", true)
                .await;
            return;
        }

        if new_password != confirm_password {
            self.Set_password_status("New passwords do not match", true)
                .await;
            return;
        }

        if new_password.len() < 4 {
            self.Set_password_status("Password must be at least 4 characters", true)
                .await;
            return;
        }

        // Authenticate current password
        match Authentication::Authenticate_user(
            Virtual_file_system::Get_instance(),
            &username,
            current_password,
        )
        .await
        {
            Ok(_) => {
                // Password is correct, proceed to change it
                match Authentication::Change_user_password(
                    Virtual_file_system::Get_instance(),
                    &username,
                    new_password,
                )
                .await
                {
                    Ok(_) => {
                        self.Set_password_status("Password changed successfully!", false)
                            .await;
                        // Clear all password fields
                        self.Clear_password_fields().await;
                    }
                    Err(_) => {
                        self.Set_password_status("Failed to change password", true)
                            .await;
                    }
                }
            }
            Err(_) => {
                self.Set_password_status("Current password is incorrect", true)
                    .await;
            }
        }
    }

    async fn Set_password_status(&mut self, message: &str, is_error: bool) {
        let message_cstr = CString::new(message).unwrap_or_default();
        unsafe {
            LVGL::lv_label_set_text(self.Password_status_label, message_cstr.as_ptr());

            // Set color based on whether it's an error or success
            if is_error {
                LVGL::lv_obj_set_style_text_color(
                    self.Password_status_label,
                    LVGL::lv_color_hex(0xFF0000), // Red for errors
                    LVGL::LV_STATE_DEFAULT,
                );
            } else {
                LVGL::lv_obj_set_style_text_color(
                    self.Password_status_label,
                    LVGL::lv_color_hex(0x00FF00), // Green for success
                    LVGL::LV_STATE_DEFAULT,
                );
            }
        }
    }

    async fn Clear_password_fields(&mut self) {
        unsafe {
            LVGL::lv_textarea_set_text(self.Current_password_text_area, c"".as_ptr());
            LVGL::lv_textarea_set_text(self.New_password_text_area, c"".as_ptr());
            LVGL::lv_textarea_set_text(self.Confirm_password_text_area, c"".as_ptr());
        }
    }
}

impl Password_tab_type {
    pub fn Create_UI(
        &mut self,
        parent_tabview: *mut LVGL::lv_obj_t,
    ) -> Result_type<*mut LVGL::lv_obj_t> {
        let tab_container =
            unsafe { LVGL::lv_tabview_add_tab(parent_tabview, c"Password".as_ptr()) };

        if tab_container.is_null() {
            return Err(crate::Error::Error_type::Failed_to_create_UI_element);
        }

        self.Tab_container = tab_container;

        // Create password change interface
        let (
            Current_password_text_area,
            New_password_text_area,
            Confirm_password_text_area,
            Change_password_button,
            Password_status_label,
        ) = unsafe {
            // Current password
            let Current_password_label = LVGL::lv_label_create(tab_container);
            LVGL::lv_label_set_text(Current_password_label, c"Current Password:".as_ptr());
            LVGL::lv_obj_align(
                Current_password_label,
                LVGL::lv_align_t_LV_ALIGN_TOP_LEFT,
                10,
                10,
            );

            let Current_password_text_area = LVGL::lv_textarea_create(tab_container);
            LVGL::lv_textarea_set_password_mode(Current_password_text_area, true);
            LVGL::lv_textarea_set_one_line(Current_password_text_area, true);
            LVGL::lv_obj_align_to(
                Current_password_text_area,
                Current_password_label,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                5,
            );

            // New password
            let New_password_label = LVGL::lv_label_create(tab_container);
            LVGL::lv_label_set_text(New_password_label, c"New Password:".as_ptr());
            LVGL::lv_obj_align_to(
                New_password_label,
                Current_password_text_area,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                20,
            );

            let New_password_text_area = LVGL::lv_textarea_create(tab_container);
            LVGL::lv_textarea_set_password_mode(New_password_text_area, true);
            LVGL::lv_textarea_set_one_line(New_password_text_area, true);
            LVGL::lv_obj_align_to(
                New_password_text_area,
                New_password_label,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                5,
            );

            // Confirm password
            let Confirm_password_label = LVGL::lv_label_create(tab_container);
            LVGL::lv_label_set_text(Confirm_password_label, c"Confirm Password:".as_ptr());
            LVGL::lv_obj_align_to(
                Confirm_password_label,
                New_password_text_area,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                20,
            );

            let Confirm_password_text_area = LVGL::lv_textarea_create(tab_container);
            LVGL::lv_textarea_set_password_mode(Confirm_password_text_area, true);
            LVGL::lv_textarea_set_one_line(Confirm_password_text_area, true);
            LVGL::lv_obj_align_to(
                Confirm_password_text_area,
                Confirm_password_label,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                5,
            );

            // Change password button
            let Change_password_button = LVGL::lv_button_create(tab_container);
            LVGL::lv_obj_align_to(
                Change_password_button,
                Confirm_password_text_area,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_MID,
                0,
                30,
            );

            let Button_label = LVGL::lv_label_create(Change_password_button);
            LVGL::lv_label_set_text(Button_label, c"Change Password".as_ptr());
            LVGL::lv_obj_center(Button_label);

            // Status label
            let Password_status_label = LVGL::lv_label_create(tab_container);
            LVGL::lv_label_set_text(Password_status_label, c"".as_ptr());
            LVGL::lv_obj_align_to(
                Password_status_label,
                Change_password_button,
                LVGL::lv_align_t_LV_ALIGN_OUT_BOTTOM_MID,
                0,
                10,
            );

            (
                Current_password_text_area,
                New_password_text_area,
                Confirm_password_text_area,
                Change_password_button,
                Password_status_label,
            )
        };

        // Store UI components
        self.Current_password_text_area = Current_password_text_area;
        self.New_password_text_area = New_password_text_area;
        self.Confirm_password_text_area = Confirm_password_text_area;
        self.Change_password_button = Change_password_button;
        self.Password_status_label = Password_status_label;

        Ok(tab_container)
    }

    pub async fn Handle_event(&mut self, event: &Event_type) -> bool {
        // Handle password change button click
        if event.Get_code() == Event_code_type::Clicked
            && event.Get_target() == self.Change_password_button
        {
            self.Handle_password_change().await;
            return true; // Event was handled
        }

        false // Event not handled by this tab
    }
}
