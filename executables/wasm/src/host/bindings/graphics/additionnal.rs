use core::ffi::CStr;

use xila::{
    graphics::{self, Color, Window, lvgl},
    log,
    task::{self},
};

use crate::host::virtual_machine::{Translator, WasmPointer};

pub unsafe fn object_delete(__translator: &mut Translator, object: WasmPointer) {
    let object = __translator.remove_host_translation(object).unwrap();
    unsafe {
        lvgl::lv_obj_delete(object as *mut _);
    }
}

pub unsafe fn window_create() -> *mut lvgl::lv_obj_t {
    task::block_on(graphics::get_instance().create_window())
        .unwrap()
        .into_raw()
}

pub unsafe fn window_pop_event(
    __translator: &mut Translator,
    window: *mut lvgl::lv_obj_t,
    code: *mut u32,
    target: *mut WasmPointer,
) {
    let mut window = unsafe { graphics::Window::from_raw(window) };

    if let Some(event) = window.pop_event() {
        unsafe {
            *code = event.code as u32;

            *target = __translator
                .translate_to_guest(event.target, false)
                .unwrap_or(0);
        }
    }

    core::mem::forget(window);
}

pub unsafe fn window_get_event_code(window: *mut lvgl::lv_obj_t) -> u32 {
    let window = unsafe { graphics::Window::from_raw(window) };

    let code = if let Some(event) = window.peek_event() {
        event.code as u32
    } else {
        graphics::EventKind::All as u32
    };

    core::mem::forget(window);

    code
}

pub unsafe fn window_get_event_target(
    __translator: &mut Translator,
    window: *mut lvgl::lv_obj_t,
) -> WasmPointer {
    let window = unsafe { graphics::Window::from_raw(window) };

    let target = if let Some(event) = window.peek_event() {
        event.target
    } else {
        log::warning!("No event available for the window");
        core::ptr::null_mut()
    };

    core::mem::forget(window);

    unsafe { __translator.translate_to_guest(target, false).unwrap() }
}

pub unsafe fn window_next_event(window: *mut lvgl::lv_obj_t) {
    let mut window = unsafe { Window::from_raw(window) };

    window.pop_event();

    core::mem::forget(window);
}

pub unsafe fn window_set_icon(
    window: *mut lvgl::lv_obj_t,
    icon_string: *const core::ffi::c_char,
    icon_color: lvgl::lv_color_t,
) {
    let mut window = unsafe { Window::from_raw(window) };

    let icon_string = unsafe { CStr::from_ptr(icon_string).to_str().unwrap() };

    let icon_color = Color::from_lvgl_color(icon_color);
    window.set_icon(icon_string, icon_color);

    core::mem::forget(window);
}

pub unsafe fn percentage(value: i32) -> i32 {
    unsafe { lvgl::lv_pct(value) }
}
