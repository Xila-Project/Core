use crate::error::Result;
use alloc::{vec, vec::Vec};
use core::ptr::null_mut;
use embedded_io::Write as _;
use xila::{
    about,
    graphics::{
        Event,
        lvgl::{self, lv_pct},
    },
    internationalization::{self, translate},
};

const TABLE_ROWS: usize = 6;

pub struct AboutTab {
    container: *mut lvgl::lv_obj_t,
}

impl AboutTab {
    pub fn new() -> Self {
        Self {
            container: null_mut() as *mut _,
        }
    }

    fn str_to_cstring(buffer: &mut Vec<u8>, s: &str) -> Result<*const i8> {
        buffer.clear();
        write!(buffer, "{}\0", s)?;
        Ok(buffer.as_ptr() as *const i8)
    }

    pub fn create_ui(
        &mut self,
        parent_tabview: *mut lvgl::lv_obj_t,
    ) -> Result<*mut lvgl::lv_obj_t> {
        self.container =
            unsafe { lvgl::lv_tabview_add_tab(parent_tabview, translate!(c"About").as_ptr()) };

        if self.container.is_null() {
            return Err(crate::error::Error::FailedToCreateUiElement);
        }

        let table = unsafe {
            let table = lvgl::lv_table_create(self.container);

            if table.is_null() {
                return Err(crate::error::Error::FailedToCreateUiElement);
            }

            lvgl::lv_obj_align(table, lvgl::lv_align_t_LV_ALIGN_CENTER, 0, 0);
            lvgl::lv_table_set_row_count(table, TABLE_ROWS as _);
            lvgl::lv_table_set_column_count(table, 2);
            lvgl::lv_obj_set_height(table, lv_pct(100));
            lvgl::lv_table_set_column_width(table, 0, 100);
            lvgl::lv_table_set_column_width(table, 1, 200);

            table
        };

        unsafe {
            let mut buffer = vec![];

            lvgl::lv_table_set_cell_value(table, 0, 0, c"Xila".as_ptr());
            lvgl::lv_table_set_cell_value(
                table,
                0,
                1,
                Self::str_to_cstring(&mut buffer, about::get_description())?,
            );

            lvgl::lv_table_set_cell_value(table, 1, 0, translate!(c"Developed by:").as_ptr());
            lvgl::lv_table_set_cell_value(
                table,
                1,
                1,
                Self::str_to_cstring(&mut buffer, about::get_authors())?,
            );

            lvgl::lv_table_set_cell_value(table, 2, 0, translate!(c"License:").as_ptr());
            lvgl::lv_table_set_cell_value(
                table,
                2,
                1,
                Self::str_to_cstring(&mut buffer, about::get_license())?,
            );

            lvgl::lv_table_set_cell_value(table, 3, 0, translate!(c"Version:").as_ptr());
            lvgl::lv_table_set_cell_value(
                table,
                3,
                1,
                Self::str_to_cstring(&mut buffer, about::get_version_string())?,
            );

            lvgl::lv_table_set_cell_value(table, 4, 0, translate!(c"Locale:").as_ptr());
            lvgl::lv_table_set_cell_value(
                table,
                4,
                1,
                Self::str_to_cstring(&mut buffer, internationalization::get_locale())?,
            );

            lvgl::lv_table_set_cell_value(table, 5, 0, translate!(c"Fallback:").as_ptr());
            lvgl::lv_table_set_cell_value(
                table,
                5,
                1,
                Self::str_to_cstring(&mut buffer, internationalization::get_fallback_locale())?,
            );
        }

        Ok(self.container)
    }

    pub async fn handle_event(&mut self, _event: &Event) -> bool {
        // For now, the about tab doesn't handle any specific events
        false
    }
}
