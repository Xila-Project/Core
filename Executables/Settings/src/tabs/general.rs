use crate::error::Result_type;
use graphics::{lvgl, Event_type};

pub struct General_tab_type {
    tab_container: *mut lvgl::lv_obj_t,
}

impl General_tab_type {
    pub fn new() -> Self {
        Self {
            tab_container: core::ptr::null_mut(),
        }
    }

    pub fn create_ui(
        &mut self,
        parent_tabview: *mut lvgl::lv_obj_t,
    ) -> Result_type<*mut lvgl::lv_obj_t> {
        let tab_container =
            unsafe { lvgl::lv_tabview_add_tab(parent_tabview, c"General".as_ptr()) };

        if tab_container.is_null() {
            return Err(crate::error::Error_type::Failed_to_create_UI_element);
        }

        self.tab_container = tab_container;

        // Create general settings UI here
        unsafe {
            let info_label = lvgl::lv_label_create(tab_container);
            lvgl::lv_label_set_text(
                info_label,
                c"General settings will be implemented here.".as_ptr(),
            );
            lvgl::lv_obj_align(info_label, lvgl::lv_align_t_LV_ALIGN_CENTER, 0, 0);
        }

        Ok(tab_container)
    }

    pub async fn Handle_event(&mut self, _event: &Event_type) -> bool {
        // For now, the general tab doesn't handle any specific events
        false
    }
}
