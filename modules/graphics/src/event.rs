use crate::{Key, lvgl};

#[derive(Debug, Clone)]
pub struct Event {
    code: EventKind,
    target: *mut lvgl::lv_obj_t,
    key: Option<Key>,
}

impl Event {
    pub fn new(code: EventKind, target: *mut lvgl::lv_obj_t, key: Option<Key>) -> Self {
        Self { code, target, key }
    }

    pub fn get_code(&self) -> EventKind {
        self.code
    }

    pub fn get_target(&self) -> *mut lvgl::lv_obj_t {
        self.target
    }

    pub fn get_key(&self) -> Option<Key> {
        self.key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EventKind {
    All = lvgl::lv_event_code_t_LV_EVENT_ALL as u16,
    Pressed = lvgl::lv_event_code_t_LV_EVENT_PRESSED as u16,
    Pressing = lvgl::lv_event_code_t_LV_EVENT_PRESSING as u16,
    PressLost = lvgl::lv_event_code_t_LV_EVENT_PRESS_LOST as u16,
    ShortClicked = lvgl::lv_event_code_t_LV_EVENT_SHORT_CLICKED as u16,
    SingleClicked = lvgl::lv_event_code_t_LV_EVENT_SINGLE_CLICKED as u16,
    DoubleClicked = lvgl::lv_event_code_t_LV_EVENT_DOUBLE_CLICKED as u16,
    TripleClicked = lvgl::lv_event_code_t_LV_EVENT_TRIPLE_CLICKED as u16,
    LongPressed = lvgl::lv_event_code_t_LV_EVENT_LONG_PRESSED as u16,
    LongPressedRepeat = lvgl::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT as u16,
    Clicked = lvgl::lv_event_code_t_LV_EVENT_CLICKED as u16,
    Released = lvgl::lv_event_code_t_LV_EVENT_RELEASED as u16,
    ScrollBegin = lvgl::lv_event_code_t_LV_EVENT_SCROLL_BEGIN as u16,
    ScrollThrowBegin = lvgl::lv_event_code_t_LV_EVENT_SCROLL_THROW_BEGIN as u16,
    ScrollEnd = lvgl::lv_event_code_t_LV_EVENT_SCROLL_END as u16,
    Scroll = lvgl::lv_event_code_t_LV_EVENT_SCROLL as u16,
    Gesture = lvgl::lv_event_code_t_LV_EVENT_GESTURE as u16,
    Key = lvgl::lv_event_code_t_LV_EVENT_KEY as u16,
    Rotary = lvgl::lv_event_code_t_LV_EVENT_ROTARY as u16,
    Focused = lvgl::lv_event_code_t_LV_EVENT_FOCUSED as u16,
    Defocused = lvgl::lv_event_code_t_LV_EVENT_DEFOCUSED as u16,
    Leave = lvgl::lv_event_code_t_LV_EVENT_LEAVE as u16,
    HitTest = lvgl::lv_event_code_t_LV_EVENT_HIT_TEST as u16,
    InputDeviceReset = lvgl::lv_event_code_t_LV_EVENT_INDEV_RESET as u16,
    HoverOver = lvgl::lv_event_code_t_LV_EVENT_HOVER_OVER as u16,
    HoverLeave = lvgl::lv_event_code_t_LV_EVENT_HOVER_LEAVE as u16,
    CoverCheck = lvgl::lv_event_code_t_LV_EVENT_COVER_CHECK as u16,
    RefreshExtDrawSize = lvgl::lv_event_code_t_LV_EVENT_REFR_EXT_DRAW_SIZE as u16,
    DrawMainBegin = lvgl::lv_event_code_t_LV_EVENT_DRAW_MAIN_BEGIN as u16,
    DrawMain = lvgl::lv_event_code_t_LV_EVENT_DRAW_MAIN as u16,
    DrawMainEnd = lvgl::lv_event_code_t_LV_EVENT_DRAW_MAIN_END as u16,
    DrawPostBegin = lvgl::lv_event_code_t_LV_EVENT_DRAW_POST_BEGIN as u16,
    DrawPost = lvgl::lv_event_code_t_LV_EVENT_DRAW_POST as u16,
    DrawPostEnd = lvgl::lv_event_code_t_LV_EVENT_DRAW_POST_END as u16,
    DrawTaskAdded = lvgl::lv_event_code_t_LV_EVENT_DRAW_TASK_ADDED as u16,
    ValueChanged = lvgl::lv_event_code_t_LV_EVENT_VALUE_CHANGED as u16,
    Insert = lvgl::lv_event_code_t_LV_EVENT_INSERT as u16,
    Refresh = lvgl::lv_event_code_t_LV_EVENT_REFRESH as u16,
    Ready = lvgl::lv_event_code_t_LV_EVENT_READY as u16,
    Cancel = lvgl::lv_event_code_t_LV_EVENT_CANCEL as u16,
    Create = lvgl::lv_event_code_t_LV_EVENT_CREATE as u16,
    Delete = lvgl::lv_event_code_t_LV_EVENT_DELETE as u16,
    ChildChanged = lvgl::lv_event_code_t_LV_EVENT_CHILD_CHANGED as u16,
    ChildCreated = lvgl::lv_event_code_t_LV_EVENT_CHILD_CREATED as u16,
    ChildDeleted = lvgl::lv_event_code_t_LV_EVENT_CHILD_DELETED as u16,
    ScreenUnloadStart = lvgl::lv_event_code_t_LV_EVENT_SCREEN_UNLOAD_START as u16,
    ScreenLoadStart = lvgl::lv_event_code_t_LV_EVENT_SCREEN_LOAD_START as u16,
    ScreenLoaded = lvgl::lv_event_code_t_LV_EVENT_SCREEN_LOADED as u16,
    ScreenUnloaded = lvgl::lv_event_code_t_LV_EVENT_SCREEN_UNLOADED as u16,
    SizeChanged = lvgl::lv_event_code_t_LV_EVENT_SIZE_CHANGED as u16,
    StyleChanged = lvgl::lv_event_code_t_LV_EVENT_STYLE_CHANGED as u16,
    LayoutChanged = lvgl::lv_event_code_t_LV_EVENT_LAYOUT_CHANGED as u16,
    GetSelfSize = lvgl::lv_event_code_t_LV_EVENT_GET_SELF_SIZE as u16,
    InvalidateArea = lvgl::lv_event_code_t_LV_EVENT_INVALIDATE_AREA as u16,
    ResolutionChanged = lvgl::lv_event_code_t_LV_EVENT_RESOLUTION_CHANGED as u16,
    ColorFormatChanged = lvgl::lv_event_code_t_LV_EVENT_COLOR_FORMAT_CHANGED as u16,
    RefreshRequest = lvgl::lv_event_code_t_LV_EVENT_REFR_REQUEST as u16,
    RefreshStart = lvgl::lv_event_code_t_LV_EVENT_REFR_START as u16,
    RefreshReady = lvgl::lv_event_code_t_LV_EVENT_REFR_READY as u16,
    RenderStart = lvgl::lv_event_code_t_LV_EVENT_RENDER_START as u16,
    RenderReady = lvgl::lv_event_code_t_LV_EVENT_RENDER_READY as u16,
    FlushStart = lvgl::lv_event_code_t_LV_EVENT_FLUSH_START as u16,
    FlushFinish = lvgl::lv_event_code_t_LV_EVENT_FLUSH_FINISH as u16,
    FlushWaitStart = lvgl::lv_event_code_t_LV_EVENT_FLUSH_WAIT_START as u16,
    FlushWaitFinish = lvgl::lv_event_code_t_LV_EVENT_FLUSH_WAIT_FINISH as u16,
    VerticalSynchronization = lvgl::lv_event_code_t_LV_EVENT_VSYNC as u16,
    Last = lvgl::lv_event_code_t_LV_EVENT_LAST as u16,
    Custom1,
    Custom2,
    Custom3,
    Custom4,
    Custom5,
    Custom6,
    Custom7,
    Custom8,
    Custom9,
    Custom10,
    Custom11,
    Custom12,
    Custom13,
    Custom14,
    Custom15,
    Custom16,
    Custom17,
    Custom18,
    Custom19,
    Custom20,
    Custom21,
    Custom22,
    Custom23,
    Custom24,
    Custom25,
    Custom26,
    Custom27,
    Custom28,
    Custom29,
    Custom30,
    Custom31,
    Custom32,
    Preprocess = lvgl::lv_event_code_t_LV_EVENT_PREPROCESS as u16,
}

impl EventKind {
    pub const fn into_lvgl_code(self) -> lvgl::lv_event_code_t {
        self as lvgl::lv_event_code_t
    }

    pub const fn from_lvgl_code(code: lvgl::lv_event_code_t) -> Self {
        unsafe { core::mem::transmute(code as u16) }
    }
}

impl From<EventKind> for lvgl::lv_event_code_t {
    fn from(code: EventKind) -> Self {
        code.into_lvgl_code()
    }
}

impl From<lvgl::lv_event_code_t> for EventKind {
    fn from(code: lvgl::lv_event_code_t) -> Self {
        EventKind::from_lvgl_code(code)
    }
}
