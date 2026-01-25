use crate::error::Result;
use alloc::{ffi::CString, format};
use core::{ffi::CStr, ptr::null_mut};
use xila::{
    about,
    graphics::{
        Event,
        lvgl::{self},
    },
    internationalization::{self, translate},
    memory,
    shared::{BYTES_SUFFIX, Unit},
};

pub struct AboutTab {
    container: *mut lvgl::lv_obj_t,
    list: *mut lvgl::lv_obj_t,
}

impl AboutTab {
    pub fn new() -> Self {
        Self {
            container: null_mut() as *mut _,
            list: null_mut() as *mut _,
        }
    }

    pub async fn create_ui(
        &mut self,
        parent_tabview: *mut lvgl::lv_obj_t,
    ) -> Result<*mut lvgl::lv_obj_t> {
        self.container =
            unsafe { lvgl::lv_tabview_add_tab(parent_tabview, translate!(c"About").as_ptr()) };

        if self.container.is_null() {
            return Err(crate::error::Error::FailedToCreateUiElement);
        }

        // Create list
        unsafe {
            self.list = lvgl::lv_list_create(self.container);

            if self.list.is_null() {
                return Err(crate::error::Error::FailedToCreateUiElement);
            }

            // List properties - fill container
            lvgl::lv_obj_set_size(self.list, lvgl::lv_pct(100), lvgl::lv_pct(100));
            lvgl::lv_obj_set_style_pad_all(self.list, 0, lvgl::LV_STATE_DEFAULT);
        }

        // Populate items - convert CStr to String
        self.create_list_item(translate!(c"Operating System:"), c"Xila")?;

        let description = CString::new(about::get_description())
            .map_err(|_| crate::error::Error::FailedToCreateUiElement)?;
        self.create_list_item(translate!(c"Description:"), &description)?;

        let authors = CString::new(about::get_authors())
            .map_err(|_| crate::error::Error::FailedToCreateUiElement)?;
        self.create_list_item(translate!(c"Developed by:"), &authors)?;

        let license = CString::new(about::get_license())
            .map_err(|_| crate::error::Error::FailedToCreateUiElement)?;
        self.create_list_item(translate!(c"License:"), &license)?;

        let version = CString::new(about::get_version_string())
            .map_err(|_| crate::error::Error::FailedToCreateUiElement)?;
        self.create_list_item(translate!(c"Version:"), &version)?;

        let locale = CString::new(format!(
            "{} ({})",
            internationalization::get_locale(),
            internationalization::get_fallback_locale(),
        ))?;
        self.create_list_item(translate!(c"Locale:"), &locale)?;

        let memory = memory::get_instance().get_total_size();
        let memory = Unit::new(memory as f32, BYTES_SUFFIX.symbol);
        let memory = CString::new(format!("{}", memory))
            .map_err(|_| crate::error::Error::FailedToCreateUiElement)?;
        self.create_list_item(translate!(c"Memory:"), &memory)?;

        Ok(self.container)
    }

    fn create_list_item(&mut self, name: &CStr, value: &CStr) -> Result<()> {
        unsafe {
            lvgl::lv_list_add_text(self.list, name.as_ptr());

            let button = lvgl::lv_list_add_button(self.list, core::ptr::null(), value.as_ptr());

            if button.is_null() {
                return Err(crate::error::Error::FailedToCreateUiElement);
            }
        }

        Ok(())
    }

    pub async fn handle_event(&mut self, _event: &Event) -> bool {
        // For now, the about tab doesn't handle any specific events
        false
    }
}
