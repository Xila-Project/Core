use crate::{Key_type, LVGL};

#[derive(Debug, Clone)]
pub struct Event_type {
    code: Event_code_type,
    target: *mut LVGL::lv_obj_t,
    key: Option<Key_type>,
}

impl Event_type {
    pub fn new(code: Event_code_type, Target: *mut LVGL::lv_obj_t, Key: Option<Key_type>) -> Self {
        Self {
            code,
            target: Target,
            key: Key,
        }
    }

    pub fn get_code(&self) -> Event_code_type {
        self.code
    }

    pub fn get_target(&self) -> *mut LVGL::lv_obj_t {
        self.target
    }

    pub fn get_key(&self) -> Option<Key_type> {
        self.key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Event_code_type {
    All = LVGL::lv_event_code_t_LV_EVENT_ALL as u16,
    Pressed = LVGL::lv_event_code_t_LV_EVENT_PRESSED as u16,
    Pressing = LVGL::lv_event_code_t_LV_EVENT_PRESSING as u16,
    Press_lost = LVGL::lv_event_code_t_LV_EVENT_PRESS_LOST as u16,
    Short_clicked = LVGL::lv_event_code_t_LV_EVENT_SHORT_CLICKED as u16,
    Long_pressed = LVGL::lv_event_code_t_LV_EVENT_LONG_PRESSED as u16,
    Long_pressed_repeat = LVGL::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT as u16,
    Clicked = LVGL::lv_event_code_t_LV_EVENT_CLICKED as u16,
    Released = LVGL::lv_event_code_t_LV_EVENT_RELEASED as u16,
    Scroll_begin = LVGL::lv_event_code_t_LV_EVENT_SCROLL_BEGIN as u16,
    Scroll_throw_begin = LVGL::lv_event_code_t_LV_EVENT_SCROLL_THROW_BEGIN as u16,
    Scroll_end = LVGL::lv_event_code_t_LV_EVENT_SCROLL_END as u16,
    Scroll = LVGL::lv_event_code_t_LV_EVENT_SCROLL as u16,
    Gesture = LVGL::lv_event_code_t_LV_EVENT_GESTURE as u16,
    Key = LVGL::lv_event_code_t_LV_EVENT_KEY as u16,
    Rotary = LVGL::lv_event_code_t_LV_EVENT_ROTARY as u16,
    Focused = LVGL::lv_event_code_t_LV_EVENT_FOCUSED as u16,
    Defocused = LVGL::lv_event_code_t_LV_EVENT_DEFOCUSED as u16,
    Leave = LVGL::lv_event_code_t_LV_EVENT_LEAVE as u16,
    Hit_test = LVGL::lv_event_code_t_LV_EVENT_HIT_TEST as u16,
    Input_device_reset = LVGL::lv_event_code_t_LV_EVENT_INDEV_RESET as u16,
    Hover_over = LVGL::lv_event_code_t_LV_EVENT_HOVER_OVER as u16,
    Hover_leave = LVGL::lv_event_code_t_LV_EVENT_HOVER_LEAVE as u16,
    Cover_check = LVGL::lv_event_code_t_LV_EVENT_COVER_CHECK as u16,
    Refresh_ext_draw_size = LVGL::lv_event_code_t_LV_EVENT_REFR_EXT_DRAW_SIZE as u16,
    Draw_main_begin = LVGL::lv_event_code_t_LV_EVENT_DRAW_MAIN_BEGIN as u16,
    Draw_main = LVGL::lv_event_code_t_LV_EVENT_DRAW_MAIN as u16,
    Draw_main_end = LVGL::lv_event_code_t_LV_EVENT_DRAW_MAIN_END as u16,
    Draw_post_begin = LVGL::lv_event_code_t_LV_EVENT_DRAW_POST_BEGIN as u16,
    Draw_post = LVGL::lv_event_code_t_LV_EVENT_DRAW_POST as u16,
    Draw_post_end = LVGL::lv_event_code_t_LV_EVENT_DRAW_POST_END as u16,
    Draw_task_added = LVGL::lv_event_code_t_LV_EVENT_DRAW_TASK_ADDED as u16,
    Value_changed = LVGL::lv_event_code_t_LV_EVENT_VALUE_CHANGED as u16,
    Insert = LVGL::lv_event_code_t_LV_EVENT_INSERT as u16,
    Refresh = LVGL::lv_event_code_t_LV_EVENT_REFRESH as u16,
    Ready = LVGL::lv_event_code_t_LV_EVENT_READY as u16,
    Cancel = LVGL::lv_event_code_t_LV_EVENT_CANCEL as u16,
    Create = LVGL::lv_event_code_t_LV_EVENT_CREATE as u16,
    Delete = LVGL::lv_event_code_t_LV_EVENT_DELETE as u16,
    Child_changed = LVGL::lv_event_code_t_LV_EVENT_CHILD_CHANGED as u16,
    Child_created = LVGL::lv_event_code_t_LV_EVENT_CHILD_CREATED as u16,
    Child_deleted = LVGL::lv_event_code_t_LV_EVENT_CHILD_DELETED as u16,
    Screen_unload_start = LVGL::lv_event_code_t_LV_EVENT_SCREEN_UNLOAD_START as u16,
    Screen_load_start = LVGL::lv_event_code_t_LV_EVENT_SCREEN_LOAD_START as u16,
    Screen_loaded = LVGL::lv_event_code_t_LV_EVENT_SCREEN_LOADED as u16,
    Screen_unloaded = LVGL::lv_event_code_t_LV_EVENT_SCREEN_UNLOADED as u16,
    Size_changed = LVGL::lv_event_code_t_LV_EVENT_SIZE_CHANGED as u16,
    Style_changed = LVGL::lv_event_code_t_LV_EVENT_STYLE_CHANGED as u16,
    Layout_changed = LVGL::lv_event_code_t_LV_EVENT_LAYOUT_CHANGED as u16,
    get_self_size = LVGL::lv_event_code_t_LV_EVENT_GET_SELF_SIZE as u16,
    Invalidate_area = LVGL::lv_event_code_t_LV_EVENT_INVALIDATE_AREA as u16,
    Resolution_changed = LVGL::lv_event_code_t_LV_EVENT_RESOLUTION_CHANGED as u16,
    Color_format_changed = LVGL::lv_event_code_t_LV_EVENT_COLOR_FORMAT_CHANGED as u16,
    Refresh_request = LVGL::lv_event_code_t_LV_EVENT_REFR_REQUEST as u16,
    Refresh_start = LVGL::lv_event_code_t_LV_EVENT_REFR_START as u16,
    Refresh_ready = LVGL::lv_event_code_t_LV_EVENT_REFR_READY as u16,
    Render_start = LVGL::lv_event_code_t_LV_EVENT_RENDER_START as u16,
    Render_ready = LVGL::lv_event_code_t_LV_EVENT_RENDER_READY as u16,
    Flush_start = LVGL::lv_event_code_t_LV_EVENT_FLUSH_START as u16,
    Flush_finish = LVGL::lv_event_code_t_LV_EVENT_FLUSH_FINISH as u16,
    Flush_wait_start = LVGL::lv_event_code_t_LV_EVENT_FLUSH_WAIT_START as u16,
    Flush_wait_finish = LVGL::lv_event_code_t_LV_EVENT_FLUSH_WAIT_FINISH as u16,
    Vertical_synchronization = LVGL::lv_event_code_t_LV_EVENT_VSYNC as u16,
    Last = LVGL::lv_event_code_t_LV_EVENT_LAST as u16,
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
    Preprocess = LVGL::lv_event_code_t_LV_EVENT_PREPROCESS as u16,
}

impl Event_code_type {
    pub const fn into_lvgl_code(self) -> LVGL::lv_event_code_t {
        self as LVGL::lv_event_code_t
    }

    pub const fn From_LVGL_code(Code: LVGL::lv_event_code_t) -> Self {
        unsafe { core::mem::transmute(Code as u16) }
    }
}

impl From<Event_code_type> for LVGL::lv_event_code_t {
    fn from(code: Event_code_type) -> Self {
        code.into_lvgl_code()
    }
}

impl From<LVGL::lv_event_code_t> for Event_code_type {
    fn from(code: LVGL::lv_event_code_t) -> Self {
        Event_code_type::From_LVGL_code(code)
    }
}
