use crate::{lvgl, Key_type};

#[derive(Debug, Clone)]
pub struct Event_type {
    code: Event_code_type,
    target: *mut lvgl::lv_obj_t,
    key: Option<Key_type>,
}

impl Event_type {
    pub fn new(code: Event_code_type, Target: *mut lvgl::lv_obj_t, Key: Option<Key_type>) -> Self {
        Self {
            code,
            target: Target,
            key: Key,
        }
    }

    pub fn get_code(&self) -> Event_code_type {
        self.code
    }

    pub fn get_target(&self) -> *mut lvgl::lv_obj_t {
        self.target
    }

    pub fn get_key(&self) -> Option<Key_type> {
        self.key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Event_code_type {
    All = lvgl::lv_event_code_t_LV_EVENT_ALL as u16,
    Pressed = lvgl::lv_event_code_t_LV_EVENT_PRESSED as u16,
    Pressing = lvgl::lv_event_code_t_LV_EVENT_PRESSING as u16,
    Press_lost = lvgl::lv_event_code_t_LV_EVENT_PRESS_LOST as u16,
    Short_clicked = lvgl::lv_event_code_t_LV_EVENT_SHORT_CLICKED as u16,
    Long_pressed = lvgl::lv_event_code_t_LV_EVENT_LONG_PRESSED as u16,
    Long_pressed_repeat = lvgl::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT as u16,
    Clicked = lvgl::lv_event_code_t_LV_EVENT_CLICKED as u16,
    Released = lvgl::lv_event_code_t_LV_EVENT_RELEASED as u16,
    Scroll_begin = lvgl::lv_event_code_t_LV_EVENT_SCROLL_BEGIN as u16,
    Scroll_throw_begin = lvgl::lv_event_code_t_LV_EVENT_SCROLL_THROW_BEGIN as u16,
    Scroll_end = lvgl::lv_event_code_t_LV_EVENT_SCROLL_END as u16,
    Scroll = lvgl::lv_event_code_t_LV_EVENT_SCROLL as u16,
    Gesture = lvgl::lv_event_code_t_LV_EVENT_GESTURE as u16,
    Key = lvgl::lv_event_code_t_LV_EVENT_KEY as u16,
    Rotary = lvgl::lv_event_code_t_LV_EVENT_ROTARY as u16,
    Focused = lvgl::lv_event_code_t_LV_EVENT_FOCUSED as u16,
    Defocused = lvgl::lv_event_code_t_LV_EVENT_DEFOCUSED as u16,
    Leave = lvgl::lv_event_code_t_LV_EVENT_LEAVE as u16,
    Hit_test = lvgl::lv_event_code_t_LV_EVENT_HIT_TEST as u16,
    Input_device_reset = lvgl::lv_event_code_t_LV_EVENT_INDEV_RESET as u16,
    Hover_over = lvgl::lv_event_code_t_LV_EVENT_HOVER_OVER as u16,
    Hover_leave = lvgl::lv_event_code_t_LV_EVENT_HOVER_LEAVE as u16,
    Cover_check = lvgl::lv_event_code_t_LV_EVENT_COVER_CHECK as u16,
    Refresh_ext_draw_size = lvgl::lv_event_code_t_LV_EVENT_REFR_EXT_DRAW_SIZE as u16,
    Draw_main_begin = lvgl::lv_event_code_t_LV_EVENT_DRAW_MAIN_BEGIN as u16,
    Draw_main = lvgl::lv_event_code_t_LV_EVENT_DRAW_MAIN as u16,
    Draw_main_end = lvgl::lv_event_code_t_LV_EVENT_DRAW_MAIN_END as u16,
    Draw_post_begin = lvgl::lv_event_code_t_LV_EVENT_DRAW_POST_BEGIN as u16,
    Draw_post = lvgl::lv_event_code_t_LV_EVENT_DRAW_POST as u16,
    Draw_post_end = lvgl::lv_event_code_t_LV_EVENT_DRAW_POST_END as u16,
    Draw_task_added = lvgl::lv_event_code_t_LV_EVENT_DRAW_TASK_ADDED as u16,
    Value_changed = lvgl::lv_event_code_t_LV_EVENT_VALUE_CHANGED as u16,
    Insert = lvgl::lv_event_code_t_LV_EVENT_INSERT as u16,
    Refresh = lvgl::lv_event_code_t_LV_EVENT_REFRESH as u16,
    Ready = lvgl::lv_event_code_t_LV_EVENT_READY as u16,
    Cancel = lvgl::lv_event_code_t_LV_EVENT_CANCEL as u16,
    Create = lvgl::lv_event_code_t_LV_EVENT_CREATE as u16,
    Delete = lvgl::lv_event_code_t_LV_EVENT_DELETE as u16,
    Child_changed = lvgl::lv_event_code_t_LV_EVENT_CHILD_CHANGED as u16,
    Child_created = lvgl::lv_event_code_t_LV_EVENT_CHILD_CREATED as u16,
    Child_deleted = lvgl::lv_event_code_t_LV_EVENT_CHILD_DELETED as u16,
    Screen_unload_start = lvgl::lv_event_code_t_LV_EVENT_SCREEN_UNLOAD_START as u16,
    Screen_load_start = lvgl::lv_event_code_t_LV_EVENT_SCREEN_LOAD_START as u16,
    Screen_loaded = lvgl::lv_event_code_t_LV_EVENT_SCREEN_LOADED as u16,
    Screen_unloaded = lvgl::lv_event_code_t_LV_EVENT_SCREEN_UNLOADED as u16,
    Size_changed = lvgl::lv_event_code_t_LV_EVENT_SIZE_CHANGED as u16,
    Style_changed = lvgl::lv_event_code_t_LV_EVENT_STYLE_CHANGED as u16,
    Layout_changed = lvgl::lv_event_code_t_LV_EVENT_LAYOUT_CHANGED as u16,
    get_self_size = lvgl::lv_event_code_t_LV_EVENT_GET_SELF_SIZE as u16,
    Invalidate_area = lvgl::lv_event_code_t_LV_EVENT_INVALIDATE_AREA as u16,
    Resolution_changed = lvgl::lv_event_code_t_LV_EVENT_RESOLUTION_CHANGED as u16,
    Color_format_changed = lvgl::lv_event_code_t_LV_EVENT_COLOR_FORMAT_CHANGED as u16,
    Refresh_request = lvgl::lv_event_code_t_LV_EVENT_REFR_REQUEST as u16,
    Refresh_start = lvgl::lv_event_code_t_LV_EVENT_REFR_START as u16,
    Refresh_ready = lvgl::lv_event_code_t_LV_EVENT_REFR_READY as u16,
    Render_start = lvgl::lv_event_code_t_LV_EVENT_RENDER_START as u16,
    Render_ready = lvgl::lv_event_code_t_LV_EVENT_RENDER_READY as u16,
    Flush_start = lvgl::lv_event_code_t_LV_EVENT_FLUSH_START as u16,
    Flush_finish = lvgl::lv_event_code_t_LV_EVENT_FLUSH_FINISH as u16,
    Flush_wait_start = lvgl::lv_event_code_t_LV_EVENT_FLUSH_WAIT_START as u16,
    Flush_wait_finish = lvgl::lv_event_code_t_LV_EVENT_FLUSH_WAIT_FINISH as u16,
    Vertical_synchronization = lvgl::lv_event_code_t_LV_EVENT_VSYNC as u16,
    Last = lvgl::lv_event_code_t_LV_EVENT_LAST as u16,
    Custom_1,
    Custom_2,
    Custom_3,
    Custom_4,
    Custom_5,
    Custom_6,
    Custom_7,
    Custom_8,
    Custom_9,
    Custom_10,
    Custom_11,
    Custom_12,
    Custom_13,
    Custom_14,
    Custom_15,
    Custom_16,
    Custom_17,
    Custom_18,
    Custom_19,
    Custom_20,
    Custom_21,
    Custom_22,
    Custom_23,
    Custom_24,
    Custom_25,
    Custom_26,
    Custom_27,
    Custom_28,
    Custom_29,
    Custom_30,
    Custom_31,
    Custom_32,
    Preprocess = lvgl::lv_event_code_t_LV_EVENT_PREPROCESS as u16,
}

impl Event_code_type {
    pub const fn into_lvgl_code(self) -> lvgl::lv_event_code_t {
        self as lvgl::lv_event_code_t
    }

    pub const fn From_LVGL_code(Code: lvgl::lv_event_code_t) -> Self {
        unsafe { core::mem::transmute(Code as u16) }
    }
}

impl From<Event_code_type> for lvgl::lv_event_code_t {
    fn from(code: Event_code_type) -> Self {
        code.into_lvgl_code()
    }
}

impl From<lvgl::lv_event_code_t> for Event_code_type {
    fn from(code: lvgl::lv_event_code_t) -> Self {
        Event_code_type::From_LVGL_code(code)
    }
}
