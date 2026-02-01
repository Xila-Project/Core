use crate::host::virtual_machine::Environment;
use xila::graphics;
use xila::graphics::lvgl::{lv_color_t, lv_obj_t, lv_pct};
use xila::log;
use xila::task;

pub unsafe fn window_create() -> *mut lv_obj_t {
    task::block_on(graphics::get_instance().create_window())
        .unwrap()
        .into_raw()
}

pub unsafe fn window_pop_event(window: *mut lv_obj_t, code: *mut u32, target: *mut *mut lv_obj_t) {
    let mut window = unsafe { graphics::Window::from_raw(window) };

    if let Some(event) = window.pop_event() {
        unsafe {
            *code = event.code as u32;
            *target = event.target;
        }
    }

    core::mem::forget(window);
}

pub unsafe fn window_get_event_code(window: *mut lv_obj_t) -> u32 {
    let window = unsafe { graphics::Window::from_raw(window) };

    let code = if let Some(event) = window.peek_event() {
        event.code as u32
    } else {
        graphics::EventKind::All as u32
    };

    core::mem::forget(window);

    code
}

pub unsafe fn window_get_event_target(window: *mut lv_obj_t) -> *mut lv_obj_t {
    let window = unsafe { graphics::Window::from_raw(window) };

    let target = if let Some(event) = window.peek_event() {
        event.target
    } else {
        log::warning!("No event available for the window");
        core::ptr::null_mut()
    };

    core::mem::forget(window);

    target
}

pub unsafe fn window_next_event(window: *mut lv_obj_t) {
    let mut window = unsafe { graphics::Window::from_raw(window) };

    window.pop_event();

    core::mem::forget(window);
}

pub unsafe fn window_set_icon(
    window: *mut lv_obj_t,
    icon_string: *const core::ffi::c_char,
    icon_color: lv_color_t,
) {
    let mut window = unsafe { graphics::Window::from_raw(window) };

    let icon_string = unsafe { core::ffi::CStr::from_ptr(icon_string).to_str().unwrap() };

    let icon_color = graphics::Color::from_lvgl_color(icon_color);

    window.set_icon(icon_string, icon_color);

    core::mem::forget(window);
}

pub unsafe fn percentage(value: i32) -> i32 {
    unsafe { lv_pct(value) }
}
