use crate::lvgl;

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
    fn from(value: u32) -> Key_type {
        match value {
            lvgl::lv_key_t_LV_KEY_UP => Key_type::Up,
            lvgl::lv_key_t_LV_KEY_DOWN => Key_type::Down,
            lvgl::lv_key_t_LV_KEY_RIGHT => Key_type::Right,
            lvgl::lv_key_t_LV_KEY_LEFT => Key_type::Left,
            lvgl::lv_key_t_LV_KEY_ESC => Key_type::Escape,
            lvgl::lv_key_t_LV_KEY_DEL => Key_type::Delete,
            lvgl::lv_key_t_LV_KEY_BACKSPACE => Key_type::Backspace,
            lvgl::lv_key_t_LV_KEY_ENTER => Key_type::Enter,
            lvgl::lv_key_t_LV_KEY_NEXT => Key_type::Next,
            lvgl::lv_key_t_LV_KEY_PREV => Key_type::Previous,
            lvgl::lv_key_t_LV_KEY_HOME => Key_type::Home,
            lvgl::lv_key_t_LV_KEY_END => Key_type::End,
            character => Key_type::Character(character as u8),
        }
    }
}

impl From<Key_type> for u32 {
    fn from(value: Key_type) -> u32 {
        match value {
            Key_type::Up => lvgl::lv_key_t_LV_KEY_UP,
            Key_type::Down => lvgl::lv_key_t_LV_KEY_DOWN,
            Key_type::Right => lvgl::lv_key_t_LV_KEY_RIGHT,
            Key_type::Left => lvgl::lv_key_t_LV_KEY_LEFT,
            Key_type::Escape => lvgl::lv_key_t_LV_KEY_ESC,
            Key_type::Delete => lvgl::lv_key_t_LV_KEY_DEL,
            Key_type::Backspace => lvgl::lv_key_t_LV_KEY_BACKSPACE,
            Key_type::Enter => lvgl::lv_key_t_LV_KEY_ENTER,
            Key_type::Next => lvgl::lv_key_t_LV_KEY_NEXT,
            Key_type::Previous => lvgl::lv_key_t_LV_KEY_PREV,
            Key_type::Home => lvgl::lv_key_t_LV_KEY_HOME,
            Key_type::End => lvgl::lv_key_t_LV_KEY_END,
            Key_type::Character(character) => character as u32,
        }
    }
}
