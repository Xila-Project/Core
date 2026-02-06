use core::ffi::CStr;

use alloc::vec::Vec;
use xila::{
    graphics::{self, Color, Window, lvgl},
    log,
    task::{self, TaskIdentifier},
};

use crate::host::virtual_machine::{Environment, Translator};

pub unsafe fn object_delete(
    __translation_map: &mut Translator,
    __task: TaskIdentifier,
    object: u16,
) {
    let object = __translation_map.remove(__task, object).unwrap();

    unsafe {
        lvgl::lv_obj_delete(object);
    }
}

pub unsafe fn window_create() -> *mut lvgl::lv_obj_t {
    task::block_on(graphics::get_instance().create_window())
        .unwrap()
        .into_raw()
}

pub unsafe fn window_pop_event(
    __environment: Environment,
    __translation_map: &mut TranslationMap,
    window: *mut lvgl::lv_obj_t,
    code: *mut u32,
    target: *mut u16,
) {
    let mut window = unsafe { graphics::Window::from_raw(window) };

    if let Some(event) = window.pop_event() {
        unsafe {
            *code = event.code as u32;

            *target = __translation_map.get_wasm_pointer(event.target).unwrap();
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
    __translation_map: &mut TranslationMap,
    window: *mut lvgl::lv_obj_t,
) -> u16 {
    let window = unsafe { graphics::Window::from_raw(window) };

    let target = if let Some(event) = window.peek_event() {
        event.target
    } else {
        log::warning!("No event available for the window");
        core::ptr::null_mut()
    };

    core::mem::forget(window);

    __translation_map.get_wasm_pointer(target).unwrap()
}

pub unsafe fn window_next_event(window: *mut lvgl::lv_obj_t) {
    let mut window = unsafe { Window::from_raw(window) };

    window.pop_event();

    core::mem::forget(window);
}

pub unsafe fn window_set_icon(
    __environment: Environment,
    __translation_map: &mut TranslationMap,
    __task: TaskIdentifier,
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

pub unsafe fn buttonmatrix_set_map(
    __environment: Environment,
    __translation_map: &mut TranslationMap,
    __task: TaskIdentifier,
    object: u16,
    map: *const *const i8,
) {
    let map_as_u32 = map as *const u32;

    // First pass: count entries (include terminating empty string)
    let mut count: usize = 0;
    loop {
        let raw = unsafe { *map_as_u32.add(count) };
        let ptr: *const i8 = unsafe { translate_to_host_pointer(&__environment, raw) };
        // increment to include this entry
        count += 1;
        // if this entry points to an empty string, stop counting
        if unsafe { *ptr == 0 } {
            break;
        }
    }

    // Allocate with the exact capacity to avoid bumps while pushing
    let mut v: Vec<*const i8> = Vec::with_capacity(count);

    let mut i = 0;
    while i < count {
        let val = unsafe { *map_as_u32.add(i) };
        let val: *const i8 = unsafe { translate_to_host_pointer(&__environment, val).unwrap() };
        v.push(val);
        i += 1;
    }

    let object = __translation_map
        .get_native_pointer(__task, object)
        .unwrap();
    unsafe {
        lvgl::lv_buttonmatrix_set_map(object, v.as_ptr());
    }

    core::mem::forget(v); // ! : deallocate the vector to avoid memory leaks when the button matrix map is deleted
}

pub unsafe fn percentage(value: i32) -> i32 {
    unsafe { lvgl::lv_pct(value) }
}
