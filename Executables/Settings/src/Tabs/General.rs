use crate::Error::Result_type;
use Graphics::{Event_type, LVGL};

pub struct General_tab_type {
    Tab_container: *mut LVGL::lv_obj_t,
}

impl General_tab_type {
    pub fn New() -> Self {
        Self {
            Tab_container: core::ptr::null_mut(),
        }
    }

    pub fn Create_UI(
        &mut self,
        Parent_tabview: *mut LVGL::lv_obj_t,
    ) -> Result_type<*mut LVGL::lv_obj_t> {
        let Tab_container =
            unsafe { LVGL::lv_tabview_add_tab(Parent_tabview, c"General".as_ptr()) };

        if Tab_container.is_null() {
            return Err(crate::Error::Error_type::Failed_to_create_UI_element);
        }

        self.Tab_container = Tab_container;

        // Create general settings UI here
        unsafe {
            let Info_label = LVGL::lv_label_create(Tab_container);
            LVGL::lv_label_set_text(
                Info_label,
                c"General settings will be implemented here.".as_ptr(),
            );
            LVGL::lv_obj_align(Info_label, LVGL::lv_align_t_LV_ALIGN_CENTER, 0, 0);
        }

        Ok(Tab_container)
    }

    pub async fn Handle_event(&mut self, _event: &Event_type) -> bool {
        // For now, the general tab doesn't handle any specific events
        false
    }
}
