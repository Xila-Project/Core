use std::{collections::VecDeque, mem::forget};

use crate::{Error_type, Result_type};

use lvgl_rust_sys::lv_obj_set_size;

use super::lvgl;

pub struct Window_type {
    Window: *mut lvgl::lv_obj_t,
}

impl Drop for Window_type {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_obj_delete(self.Window);
        }
    }
}

unsafe extern "C" fn Event_callback(Event: *mut lvgl::lv_event_t) {
    let Code = lvgl::lv_event_get_code(Event);

    let Queue = lvgl::lv_event_get_user_data(Event) as *mut VecDeque<Event_type>;
    let mut Queue = Box::from_raw(Queue);

    let Target = lvgl::lv_event_get_target(Event) as *mut lvgl::lv_obj_t;

    match Code {
        lvgl::lv_event_code_t_LV_EVENT_CHILD_CREATED => {
            lvgl::lv_obj_add_flag(Target, lvgl::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);
        }
        _ => {
            Queue.push_back(Event_type { Code, Target });
        }
    }

    forget(Queue); // Forget the queue to prevent it from being dropped.
}

#[derive(Debug, Clone)]
pub struct Event_type {
    pub Code: lvgl::lv_event_code_t,
    pub Target: *mut lvgl::lv_obj_t,
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
    pub unsafe fn New(Parent_object: *mut lvgl::lv_obj_t) -> Result_type<Self> {
        let Window = unsafe { lvgl::lv_obj_create(Parent_object) };

        if Window.is_null() {
            return Err(Error_type::Failed_to_create_object);
        }

        let Queue: VecDeque<Event_type> = VecDeque::with_capacity(10);
        let Queue = Box::into_raw(Box::new(Queue));

        unsafe {
            // Set the event callback for the window.
            lvgl::lv_obj_add_event_cb(
                Window,
                Some(Event_callback),
                lvgl::lv_event_code_t_LV_EVENT_ALL,
                Queue as *mut core::ffi::c_void,
            );
            lvgl::lv_obj_set_user_data(Window, Queue as *mut core::ffi::c_void);
            // Set the size of the window to 100% of the parent object.
            lv_obj_set_size(Window, lvgl::lv_pct(100), lvgl::lv_pct(100));
        }

        Ok(Self { Window })
    }

    pub fn Peek_event(&self) -> Option<Event_type> {
        let Queue = unsafe { lvgl::lv_obj_get_user_data(self.Window) as *mut VecDeque<Event_type> };

        let Queue = unsafe { Box::from_raw(Queue) };

        let Event = Queue.front().cloned();

        forget(Queue);

        Event
    }

    pub fn Pop_event(&mut self) -> Option<Event_type> {
        let Queue = unsafe { lvgl::lv_obj_get_user_data(self.Window) as *mut VecDeque<Event_type> };

        let mut Queue = unsafe { Box::from_raw(Queue) };

        let Event = Queue.pop_front();

        forget(Queue);

        Event
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
    pub unsafe fn From_raw(Window: *mut lvgl::lv_obj_t) -> Self {
        Self { Window }
    }

    pub fn Into_raw(self) -> *mut lvgl::lv_obj_t {
        let Window = self.Window;

        forget(self);

        Window
    }
}
