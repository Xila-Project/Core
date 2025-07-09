use crate::lvgl;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input_type_type {
    Pointer,
    Keypad,
}

impl From<Input_type_type> for lvgl::lv_indev_type_t {
    fn from(value: Input_type_type) -> Self {
        match value {
            Input_type_type::Pointer => lvgl::lv_indev_type_t_LV_INDEV_TYPE_POINTER,
            Input_type_type::Keypad => lvgl::lv_indev_type_t_LV_INDEV_TYPE_KEYPAD,
        }
    }
}
