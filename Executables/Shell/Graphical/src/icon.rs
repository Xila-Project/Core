use alloc::ffi::CString;
use graphics::{lvgl, Color_type, Point_type};

use crate::error::{Error_type, Result_type};

pub unsafe fn create_icon(
    parent: *mut lvgl::lv_obj_t,
    icon_color: Color_type,
    icon_string: &str,
    size: Point_type,
) -> Result_type<*mut lvgl::lv_obj_t> {
    let icon = lvgl::lv_button_create(parent);

    if icon.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    lvgl::lv_obj_set_size(icon, size.get_x().into(), size.get_y().into());
    lvgl::lv_obj_set_style_pad_all(icon, 0, lvgl::LV_STATE_DEFAULT);
    lvgl::lv_obj_set_style_border_width(icon, 0, lvgl::LV_STATE_DEFAULT);

    let radius: i32 = size.get_x() as i32 / 3;

    lvgl::lv_obj_set_style_radius(icon, radius, lvgl::LV_STATE_DEFAULT);

    lvgl::lv_obj_set_style_bg_color(icon, icon_color.into_lvgl_color(), lvgl::LV_STATE_DEFAULT);

    let label = lvgl::lv_label_create(icon);

    if label.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    if size.get_x() >= 48 {
        lvgl::lv_obj_set_style_text_font(
            label,
            &lvgl::lv_font_montserrat_28,
            lvgl::LV_STATE_DEFAULT,
        );
    } else {
        lvgl::lv_obj_set_style_text_font(
            label,
            &lvgl::lv_font_montserrat_18,
            lvgl::LV_STATE_DEFAULT,
        );
    }

    let icon_string = CString::new(icon_string).map_err(Error_type::Null_character_in_string)?;

    lvgl::lv_label_set_text(label, icon_string.as_ptr());
    lvgl::lv_obj_set_align(label, lvgl::lv_align_t_LV_ALIGN_CENTER);

    Ok(icon)
}
