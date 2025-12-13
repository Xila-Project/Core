use core::ffi::c_char;

pub use lvgl_rust_sys::*;

use crate::Point;

pub const LV_SIZE_CONTENT: i32 = (LV_COORD_MAX | LV_COORD_TYPE_SPEC) as i32;

/// Set the padding of an object on all sides
///
/// # Arguments
///
/// * `Object` - The object to set the padding of.
/// * `Padding` - The padding to set.
/// * `Selector` - The selector to set the padding for.
///
/// # Safety
///
/// This function is unsafe because it may dereference raw pointers (e.g. `Object`).
pub unsafe fn lv_obj_set_style_pad_all(
    object: *mut lv_obj_t,
    padding: i32,
    selector: lv_style_selector_t,
) {
    unsafe {
        lv_obj_set_style_pad_top(object, padding, selector);
        lv_obj_set_style_pad_bottom(object, padding, selector);
        lv_obj_set_style_pad_left(object, padding, selector);
        lv_obj_set_style_pad_right(object, padding, selector);
    }
}

/// Set the padding of an object on the top side
///
/// # Arguments
///
/// * `Object` - The object to set the padding of.
/// * `Padding` - The padding to set.
/// * `Selector` - The selector to set the padding for.
///
/// # Safety
///
/// This function is unsafe because it may dereference raw pointers (e.g. `Object`).
///
pub unsafe fn lv_obj_move_foreground(object: *mut lv_obj_t) {
    unsafe {
        lv_obj_move_to_index(object, -1);
    }
}

/// Get the size of an object as a Point
///
/// # Arguments
/// * `Object` - The object to get the size of.
///
///  # Safety
/// This function is unsafe because it may dereference raw pointers (e.g. `Object`).
///
pub unsafe fn lv_obj_get_size(object: *mut lv_obj_t) -> Point {
    unsafe {
        let width = lv_obj_get_width(object) as i16;
        let height = lv_obj_get_height(object) as i16;

        Point::new(width, height)
    }
}

pub unsafe fn lv_tabview_add_tab(tabview: *mut lv_obj_t, name: *const c_char) -> *mut lv_obj_t {
    unsafe {
        let page = lvgl_rust_sys::lv_tabview_add_tab(tabview, name);

        let bar = lv_tabview_get_tab_bar(tabview);

        lv_obj_set_size(bar, LV_SIZE_CONTENT, LV_SIZE_CONTENT);

        // get latest tab button
        let tab_count = lv_obj_get_child_count(bar);
        let button = lv_obj_get_child(bar, (tab_count - 1) as _);

        // don't make it grow
        lv_obj_set_flex_grow(button, 0);
        lv_obj_set_size(button, LV_SIZE_CONTENT, LV_SIZE_CONTENT);

        page
    }
}
