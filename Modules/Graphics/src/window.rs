use alloc::boxed::Box;

use core::{mem::forget, str};

use alloc::collections::VecDeque;

use crate::{Color_type, Error_type, Event::Event_type, Event_code_type, Result_type};

use super::LVGL;

struct User_data_type {
    pub queue: VecDeque<Event_type>,
    pub icon_text: [u8; 2],
    pub icon_color: Color_type,
}

pub struct Window_type {
    window: *mut LVGL::lv_obj_t,
}

impl Drop for Window_type {
    fn drop(&mut self) {
        unsafe {
            let user_data = LVGL::lv_obj_get_user_data(self.window) as *mut User_data_type;

            let _user_data = Box::from_raw(user_data);

            LVGL::lv_obj_delete(self.window);
        }
    }
}

unsafe extern "C" fn event_callback(event: *mut LVGL::lv_event_t) {
    let code = LVGL::lv_event_get_code(event);

    let queue = LVGL::lv_event_get_user_data(event) as *mut VecDeque<Event_type>;

    let target = LVGL::lv_event_get_target(event) as *mut LVGL::lv_obj_t;

    match code {
        LVGL::lv_event_code_t_LV_EVENT_CHILD_CREATED => {
            LVGL::lv_obj_add_flag(target, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);

            (*queue).push_back(Event_type::new(
                Event_code_type::Child_created,
                target,
                None,
            ));
        }
        LVGL::lv_event_code_t_LV_EVENT_KEY => {
            let key = unsafe { LVGL::lv_indev_get_key(LVGL::lv_indev_active()) };

            (*queue).push_back(Event_type::new(
                Event_code_type::Key,
                target,
                Some(key.into()),
            ));
        }
        _ => {
            (*queue).push_back(Event_type::new(
                Event_code_type::From_LVGL_code(code),
                target,
                None,
            ));
        }
    }
}

impl Window_type {
    /// Create a new window.
    ///
    /// # Arguments
    ///
    /// * `Parent_object` - The parent object of the window.
    ///
    /// # Returns
    ///
    /// * `Result_type<Self>` - The result of the operation.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it may dereference raw pointers (e.g. `Parent_object`).
    ///
    pub unsafe fn new(parent_object: *mut LVGL::lv_obj_t) -> Result_type<Self> {
        let window = unsafe { LVGL::lv_obj_create(parent_object) };

        if window.is_null() {
            return Err(Error_type::Failed_to_create_object);
        }

        let user_data = User_data_type {
            queue: VecDeque::with_capacity(10),
            icon_text: [b'I', b'c'],
            icon_color: Color_type::BLACK,
        };

        let mut user_data = Box::new(user_data);

        unsafe {
            // Set the event callback for the window.
            LVGL::lv_obj_add_event_cb(
                window,
                Some(event_callback),
                LVGL::lv_event_code_t_LV_EVENT_ALL,
                &mut user_data.queue as *mut _ as *mut core::ffi::c_void,
            );
            LVGL::lv_obj_set_user_data(window, Box::into_raw(user_data) as *mut core::ffi::c_void);
            // Set the size of the window to 100% of the parent object.
            LVGL::lv_obj_set_size(window, LVGL::lv_pct(100), LVGL::lv_pct(100));
            LVGL::lv_obj_set_style_border_width(window, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(window, 0, LVGL::LV_STATE_DEFAULT);
        }

        Ok(Self { window })
    }

    pub fn get_identifier(&self) -> usize {
        self.window as usize
    }

    pub fn peek_event(&self) -> Option<Event_type> {
        let user_data = unsafe { LVGL::lv_obj_get_user_data(self.window) as *mut User_data_type };

        let user_data = unsafe { Box::from_raw(user_data) };

        let event = user_data.queue.front().cloned();

        forget(user_data);

        event
    }

    pub fn pop_event(&mut self) -> Option<Event_type> {
        let user_data = unsafe { LVGL::lv_obj_get_user_data(self.window) as *mut User_data_type };

        let mut user_data = unsafe { Box::from_raw(user_data) };

        let event = user_data.queue.pop_front();

        forget(user_data);

        event
    }

    pub fn get_object(&self) -> *mut LVGL::lv_obj_t {
        self.window
    }

    pub fn get_icon(&self) -> (&str, Color_type) {
        let user_data = unsafe {
            let user_data = LVGL::lv_obj_get_user_data(self.window) as *mut User_data_type;

            &*user_data
        };

        unsafe {
            (
                str::from_utf8_unchecked(&user_data.icon_text),
                user_data.icon_color,
            )
        }
    }

    pub fn set_icon(&mut self, icon_string: &str, icon_color: Color_type) {
        let user_data = unsafe { LVGL::lv_obj_get_user_data(self.window) as *mut User_data_type };

        let user_data = unsafe { &mut *user_data };

        let mut iterator = icon_string.chars();

        if let Some(character) = iterator.next() {
            user_data.icon_text[0] = character as u8;
        }

        if let Some(character) = iterator.next() {
            user_data.icon_text[1] = character as u8;
        }

        user_data.icon_color = icon_color;
    }

    /// Convert a raw pointer to a window object.
    ///
    /// # Returns
    ///
    /// * `Window` - The raw pointer to the window.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it may dereference raw pointers (e.g. `Window`).
    ///
    pub unsafe fn from_raw(window: *mut LVGL::lv_obj_t) -> Self {
        Self { window }
    }

    pub fn into_raw(self) -> *mut LVGL::lv_obj_t {
        let window = self.window;

        forget(self);

        window
    }
}
