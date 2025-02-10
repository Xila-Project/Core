use std::ffi::CString;

use Graphics::{Color_type, Point_type, LVGL};

use crate::Error::{Error_type, Result_type};

pub unsafe fn Create_icon(
    Parent: *mut LVGL::lv_obj_t,
    Icon_color: Color_type,
    Icon_string: &str,
    Size: Point_type,
) -> Result_type<*mut LVGL::lv_obj_t> {
    let Icon = LVGL::lv_button_create(Parent);

    if Icon.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    LVGL::lv_obj_set_size(Icon, Size.Get_x().into(), Size.Get_y().into());
    LVGL::lv_obj_set_style_pad_all(Icon, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_border_width(Icon, 0, LVGL::LV_STATE_DEFAULT);

    let Radius: i32 = Size.Get_x() as i32 / 3;

    LVGL::lv_obj_set_style_radius(Icon, Radius, LVGL::LV_STATE_DEFAULT);

    LVGL::lv_obj_set_style_bg_color(Icon, Icon_color.Into_LVGL_color(), LVGL::LV_STATE_DEFAULT);

    let Label = LVGL::lv_label_create(Icon);

    if Label.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    if Size.Get_x() >= 48 {
        LVGL::lv_obj_set_style_text_font(
            Label,
            &LVGL::lv_font_montserrat_28,
            LVGL::LV_STATE_DEFAULT,
        );
    } else {
        LVGL::lv_obj_set_style_text_font(
            Label,
            &LVGL::lv_font_montserrat_18,
            LVGL::LV_STATE_DEFAULT,
        );
    }

    let Icon_string = CString::new(Icon_string).map_err(Error_type::Null_character_in_string)?;

    LVGL::lv_label_set_text(Label, Icon_string.as_ptr());
    LVGL::lv_obj_set_align(Label, LVGL::lv_align_t_LV_ALIGN_CENTER);

    Ok(Icon)
}
