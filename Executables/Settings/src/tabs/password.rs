use crate::error::Result;

use alloc::ffi::CString;
use authentication;
use graphics::{lvgl, Event, EventKind};
use task;
use users;
use virtual_file_system;

pub struct PasswordTabType {
    tab_container: *mut lvgl::lv_obj_t,
    current_password_text_area: *mut lvgl::lv_obj_t,
    new_password_text_area: *mut lvgl::lv_obj_t,
    confirm_password_text_area: *mut lvgl::lv_obj_t,
    change_password_button: *mut lvgl::lv_obj_t,
    password_status_label: *mut lvgl::lv_obj_t,
}

impl PasswordTabType {
    pub fn new() -> Self {
        Self {
            tab_container: core::ptr::null_mut(),
            current_password_text_area: core::ptr::null_mut(),
            new_password_text_area: core::ptr::null_mut(),
            confirm_password_text_area: core::ptr::null_mut(),
            change_password_button: core::ptr::null_mut(),
            password_status_label: core::ptr::null_mut(),
        }
    }

    async fn handle_password_change(&mut self) {
        // Get the current user
        let task_manager = task::get_instance();
        let current_task = task_manager.get_current_task_identifier().await;

        let user_id = match task_manager.get_user(current_task).await {
            Ok(user_id) => user_id,
            Err(_) => {
                self.set_password_status("Failed to get current user", true)
                    .await;
                return;
            }
        };

        let users_manager = users::get_instance();
        let username = match users_manager.get_user_name(user_id).await {
            Ok(name) => name,
            Err(_) => {
                self.set_password_status("Failed to get username", true)
                    .await;
                return;
            }
        };

        // Get passwords from text areas
        let (current_password, new_password, confirm_password) = unsafe {
            let current = core::ffi::CStr::from_ptr(lvgl::lv_textarea_get_text(
                self.current_password_text_area,
            ));
            let new =
                core::ffi::CStr::from_ptr(lvgl::lv_textarea_get_text(self.new_password_text_area));
            let confirm = core::ffi::CStr::from_ptr(lvgl::lv_textarea_get_text(
                self.confirm_password_text_area,
            ));

            let current = match current.to_str() {
                Ok(s) => s,
                Err(_) => {
                    self.set_password_status("Invalid characters in current password", true)
                        .await;
                    return;
                }
            };

            let new = match new.to_str() {
                Ok(s) => s,
                Err(_) => {
                    self.set_password_status("Invalid characters in new password", true)
                        .await;
                    return;
                }
            };

            let confirm = match confirm.to_str() {
                Ok(s) => s,
                Err(_) => {
                    self.set_password_status("Invalid characters in confirm password", true)
                        .await;
                    return;
                }
            };

            (current, new, confirm)
        };

        // Validate inputs
        if current_password.is_empty() {
            self.set_password_status("Current password cannot be empty", true)
                .await;
            return;
        }

        if new_password.is_empty() {
            self.set_password_status("New password cannot be empty", true)
                .await;
            return;
        }

        if new_password != confirm_password {
            self.set_password_status("New passwords do not match", true)
                .await;
            return;
        }

        if new_password.len() < 4 {
            self.set_password_status("Password must be at least 4 characters", true)
                .await;
            return;
        }

        // Authenticate current password
        match authentication::authenticate_user(
            virtual_file_system::get_instance(),
            &username,
            current_password,
        )
        .await
        {
            Ok(_) => {
                // Password is correct, proceed to change it
                match authentication::change_user_password(
                    virtual_file_system::get_instance(),
                    &username,
                    new_password,
                )
                .await
                {
                    Ok(_) => {
                        self.set_password_status("Password changed successfully!", false)
                            .await;
                        // Clear all password fields
                        self.clear_password_fields().await;
                    }
                    Err(_) => {
                        self.set_password_status("Failed to change password", true)
                            .await;
                    }
                }
            }
            Err(_) => {
                self.set_password_status("Current password is incorrect", true)
                    .await;
            }
        }
    }

    async fn set_password_status(&mut self, message: &str, is_error: bool) {
        let message_cstr = CString::new(message).unwrap_or_default();
        unsafe {
            lvgl::lv_label_set_text(self.password_status_label, message_cstr.as_ptr());

            // Set color based on whether it's an error or success
            if is_error {
                lvgl::lv_obj_set_style_text_color(
                    self.password_status_label,
                    lvgl::lv_color_hex(0xFF0000), // Red for errors
                    lvgl::LV_STATE_DEFAULT,
                );
            } else {
                lvgl::lv_obj_set_style_text_color(
                    self.password_status_label,
                    lvgl::lv_color_hex(0x00FF00), // Green for success
                    lvgl::LV_STATE_DEFAULT,
                );
            }
        }
    }

    async fn clear_password_fields(&mut self) {
        unsafe {
            lvgl::lv_textarea_set_text(self.current_password_text_area, c"".as_ptr());
            lvgl::lv_textarea_set_text(self.new_password_text_area, c"".as_ptr());
            lvgl::lv_textarea_set_text(self.confirm_password_text_area, c"".as_ptr());
        }
    }
}

impl PasswordTabType {
    pub fn create_ui(
        &mut self,
        parent_tabview: *mut lvgl::lv_obj_t,
    ) -> Result<*mut lvgl::lv_obj_t> {
        let tab_container =
            unsafe { lvgl::lv_tabview_add_tab(parent_tabview, c"Password".as_ptr()) };

        if tab_container.is_null() {
            return Err(crate::error::Error::FailedToCreateUiElement);
        }

        self.tab_container = tab_container;

        // Create password change interface
        let (
            current_password_text_area,
            new_password_text_area,
            confirm_password_text_area,
            change_password_button,
            password_status_label,
        ) = unsafe {
            // Current password
            let current_password_label = lvgl::lv_label_create(tab_container);
            lvgl::lv_label_set_text(current_password_label, c"Current Password:".as_ptr());
            lvgl::lv_obj_align(
                current_password_label,
                lvgl::lv_align_t_LV_ALIGN_TOP_LEFT,
                10,
                10,
            );

            let current_password_text_area = lvgl::lv_textarea_create(tab_container);
            lvgl::lv_textarea_set_password_mode(current_password_text_area, true);
            lvgl::lv_textarea_set_one_line(current_password_text_area, true);
            lvgl::lv_obj_align_to(
                current_password_text_area,
                current_password_label,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                5,
            );

            // New password
            let new_password_label = lvgl::lv_label_create(tab_container);
            lvgl::lv_label_set_text(new_password_label, c"New Password:".as_ptr());
            lvgl::lv_obj_align_to(
                new_password_label,
                current_password_text_area,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                20,
            );

            let new_password_text_area = lvgl::lv_textarea_create(tab_container);
            lvgl::lv_textarea_set_password_mode(new_password_text_area, true);
            lvgl::lv_textarea_set_one_line(new_password_text_area, true);
            lvgl::lv_obj_align_to(
                new_password_text_area,
                new_password_label,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                5,
            );

            // Confirm password
            let confirm_password_label = lvgl::lv_label_create(tab_container);
            lvgl::lv_label_set_text(confirm_password_label, c"Confirm Password:".as_ptr());
            lvgl::lv_obj_align_to(
                confirm_password_label,
                new_password_text_area,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                20,
            );

            let confirm_password_text_area = lvgl::lv_textarea_create(tab_container);
            lvgl::lv_textarea_set_password_mode(confirm_password_text_area, true);
            lvgl::lv_textarea_set_one_line(confirm_password_text_area, true);
            lvgl::lv_obj_align_to(
                confirm_password_text_area,
                confirm_password_label,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_LEFT,
                0,
                5,
            );

            // Change password button
            let change_password_button = lvgl::lv_button_create(tab_container);
            lvgl::lv_obj_align_to(
                change_password_button,
                confirm_password_text_area,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_MID,
                0,
                30,
            );

            let button_label = lvgl::lv_label_create(change_password_button);
            lvgl::lv_label_set_text(button_label, c"Change Password".as_ptr());
            lvgl::lv_obj_center(button_label);

            // Status label
            let password_status_label = lvgl::lv_label_create(tab_container);
            lvgl::lv_label_set_text(password_status_label, c"".as_ptr());
            lvgl::lv_obj_align_to(
                password_status_label,
                change_password_button,
                lvgl::lv_align_t_LV_ALIGN_OUT_BOTTOM_MID,
                0,
                10,
            );

            (
                current_password_text_area,
                new_password_text_area,
                confirm_password_text_area,
                change_password_button,
                password_status_label,
            )
        };

        // Store UI components
        self.current_password_text_area = current_password_text_area;
        self.new_password_text_area = new_password_text_area;
        self.confirm_password_text_area = confirm_password_text_area;
        self.change_password_button = change_password_button;
        self.password_status_label = password_status_label;

        Ok(tab_container)
    }

    pub async fn handle_event(&mut self, event: &Event) -> bool {
        // Handle password change button click
        if event.get_code() == EventKind::Clicked
            && event.get_target() == self.change_password_button
        {
            self.handle_password_change().await;
            return true; // Event was handled
        }

        false // Event not handled by this tab
    }
}
