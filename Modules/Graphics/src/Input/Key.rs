use crate::LVGL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key_type {
    Up,
    Down,
    Right,
    Left,
    Escape,
    Delete,
    Backspace,
    Enter,
    Next,
    Previous,
    Home,
    End,
    Character(u8),
}

impl From<u32> for Key_type {
    fn from(Value: u32) -> Key_type {
        match Value {
            LVGL::lv_key_t_LV_KEY_UP => Key_type::Up,
            LVGL::lv_key_t_LV_KEY_DOWN => Key_type::Down,
            LVGL::lv_key_t_LV_KEY_RIGHT => Key_type::Right,
            LVGL::lv_key_t_LV_KEY_LEFT => Key_type::Left,
            LVGL::lv_key_t_LV_KEY_ESC => Key_type::Escape,
            LVGL::lv_key_t_LV_KEY_DEL => Key_type::Delete,
            LVGL::lv_key_t_LV_KEY_BACKSPACE => Key_type::Backspace,
            LVGL::lv_key_t_LV_KEY_ENTER => Key_type::Enter,
            LVGL::lv_key_t_LV_KEY_NEXT => Key_type::Next,
            LVGL::lv_key_t_LV_KEY_PREV => Key_type::Previous,
            LVGL::lv_key_t_LV_KEY_HOME => Key_type::Home,
            LVGL::lv_key_t_LV_KEY_END => Key_type::End,
            Character => Key_type::Character(Character as u8),
        }
    }
}

impl From<Key_type> for u32 {
    fn from(Value: Key_type) -> u32 {
        match Value {
            Key_type::Up => LVGL::lv_key_t_LV_KEY_UP,
            Key_type::Down => LVGL::lv_key_t_LV_KEY_DOWN,
            Key_type::Right => LVGL::lv_key_t_LV_KEY_RIGHT,
            Key_type::Left => LVGL::lv_key_t_LV_KEY_LEFT,
            Key_type::Escape => LVGL::lv_key_t_LV_KEY_ESC,
            Key_type::Delete => LVGL::lv_key_t_LV_KEY_DEL,
            Key_type::Backspace => LVGL::lv_key_t_LV_KEY_BACKSPACE,
            Key_type::Enter => LVGL::lv_key_t_LV_KEY_ENTER,
            Key_type::Next => LVGL::lv_key_t_LV_KEY_NEXT,
            Key_type::Previous => LVGL::lv_key_t_LV_KEY_PREV,
            Key_type::Home => LVGL::lv_key_t_LV_KEY_HOME,
            Key_type::End => LVGL::lv_key_t_LV_KEY_END,
            Key_type::Character(Character) => Character as u32,
        }
    }
}
