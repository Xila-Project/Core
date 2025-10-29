use super::lvgl;
use crate::{Color, Error, EventKind, Result, event::Event};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::{mem::forget, str};

struct UserData {
    pub queue: VecDeque<Event>,
    pub icon_text: [u8; 2],
    pub icon_color: Color,
}

pub struct Window {
    window: *mut lvgl::lv_obj_t,
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            let user_data = lvgl::lv_obj_get_user_data(self.window) as *mut UserData;

            let _user_data = Box::from_raw(user_data);

            lvgl::lv_obj_delete(self.window);
        }
    }
}

unsafe extern "C" fn event_callback(event: *mut lvgl::lv_event_t) {
    unsafe {
        let code = lvgl::lv_event_get_code(event);

        let queue = lvgl::lv_event_get_user_data(event) as *mut VecDeque<Event>;

        let target = lvgl::lv_event_get_target(event) as *mut lvgl::lv_obj_t;

        match code {
            lvgl::lv_event_code_t_LV_EVENT_CHILD_CREATED => {
                lvgl::lv_obj_add_flag(target, lvgl::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);

                (*queue).push_back(Event::new(EventKind::ChildCreated, target, None));
            }
            lvgl::lv_event_code_t_LV_EVENT_KEY => {
                let key = lvgl::lv_indev_get_key(lvgl::lv_indev_active());

                (*queue).push_back(Event::new(EventKind::Key, target, Some(key.into())));
            }
            _ => {
                (*queue).push_back(Event::new(EventKind::from_lvgl_code(code), target, None));
            }
        }
    }
}

impl Window {
    /// Create a new window.
    ///
    /// # Arguments
    ///
    /// * `Parent_object` - The parent object of the window.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The result of the operation.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it may dereference raw pointers (e.g. `Parent_object`).
    ///
    pub unsafe fn new(parent_object: *mut lvgl::lv_obj_t) -> Result<Self> {
        let window = unsafe { lvgl::lv_obj_create(parent_object) };

        if window.is_null() {
            return Err(Error::FailedToCreateObject);
        }

        let user_data = UserData {
            queue: VecDeque::with_capacity(10),
            icon_text: [b'I', b'c'],
            icon_color: Color::BLACK,
        };

        let mut user_data = Box::new(user_data);

        unsafe {
            // Set the event callback for the window.
            lvgl::lv_obj_add_event_cb(
                window,
                Some(event_callback),
                lvgl::lv_event_code_t_LV_EVENT_ALL,
                &mut user_data.queue as *mut _ as *mut core::ffi::c_void,
            );
            lvgl::lv_obj_add_flag(window, lvgl::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);
            lvgl::lv_obj_set_user_data(window, Box::into_raw(user_data) as *mut core::ffi::c_void);
            // Set the size of the window to 100% of the parent object.
            lvgl::lv_obj_set_size(window, lvgl::lv_pct(100), lvgl::lv_pct(100));
            lvgl::lv_obj_set_style_border_width(window, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_radius(window, 0, lvgl::LV_STATE_DEFAULT);
        }

        Ok(Self { window })
    }

    pub fn get_identifier(&self) -> usize {
        self.window as usize
    }

    pub fn peek_event(&self) -> Option<Event> {
        let user_data = unsafe { lvgl::lv_obj_get_user_data(self.window) as *mut UserData };

        let user_data = unsafe { Box::from_raw(user_data) };

        let event = user_data.queue.front().cloned();

        forget(user_data);

        event
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        let user_data = unsafe { lvgl::lv_obj_get_user_data(self.window) as *mut UserData };

        let mut user_data = unsafe { Box::from_raw(user_data) };

        let event = user_data.queue.pop_front();

        forget(user_data);

        event
    }

    pub fn get_object(&self) -> *mut lvgl::lv_obj_t {
        self.window
    }

    pub fn get_icon(&self) -> (&str, Color) {
        let user_data = unsafe {
            let user_data = lvgl::lv_obj_get_user_data(self.window) as *mut UserData;

            &*user_data
        };

        unsafe {
            (
                str::from_utf8_unchecked(&user_data.icon_text),
                user_data.icon_color,
            )
        }
    }

    pub fn set_icon(&mut self, icon_string: &str, icon_color: Color) {
        let user_data = unsafe { lvgl::lv_obj_get_user_data(self.window) as *mut UserData };

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
    pub unsafe fn from_raw(window: *mut lvgl::lv_obj_t) -> Self {
        Self { window }
    }

    pub fn into_raw(self) -> *mut lvgl::lv_obj_t {
        let window = self.window;

        forget(self);

        window
    }
}
