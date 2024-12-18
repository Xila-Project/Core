use Graphics::LVGL;

use crate::Error::{Error_type, Result_type};

pub unsafe fn Create_icon(
    Parent: *mut LVGL::lv_obj_t,
    Name: &str,
) -> Result_type<*mut LVGL::lv_obj_t> {
    let Icon = LVGL::lv_button_create(Parent);

    if Icon.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    LVGL::lv_obj_set_size(Icon, 32, 32);
    LVGL::lv_obj_set_style_pad_all(Icon, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_border_width(Icon, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_radius(Icon, 5, LVGL::LV_STATE_DEFAULT);

    let Color = Get_color_from_name(Name);

    LVGL::lv_obj_set_style_bg_color(Icon, Color, LVGL::LV_STATE_DEFAULT);

    let Label = LVGL::lv_label_create(Icon);

    if Label.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    LVGL::lv_label_set_text(Label, c"IC".as_ptr());
    LVGL::lv_obj_set_align(Label, LVGL::lv_align_t_LV_ALIGN_CENTER);

    Ok(Icon)
}

fn Get_color_from_name(Name: &str) -> LVGL::lv_color_t {
    let RGB: [u8; 3] = Name
        .chars()
        .enumerate()
        .fold([0_u8; 3], |mut RGB, (Index, Byte)| {
            RGB[Index % 3] = RGB[Index % 3].wrapping_add(Byte as u8);

            RGB
        });

    unsafe { LVGL::lv_color_make(RGB[0], RGB[1], RGB[2]) }
}