use crate::LVGL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input_type_type {
    Pointer,
    Keypad,
}

impl From<Input_type_type> for LVGL::lv_indev_type_t {
    fn from(Value: Input_type_type) -> Self {
        match Value {
            Input_type_type::Pointer => LVGL::lv_indev_type_t_LV_INDEV_TYPE_POINTER,
            Input_type_type::Keypad => LVGL::lv_indev_type_t_LV_INDEV_TYPE_KEYPAD,
        }
    }
}
