pub use lvgl_rust_sys::*;

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
    lv_obj_set_style_pad_top(object, padding, selector);
    lv_obj_set_style_pad_bottom(object, padding, selector);
    lv_obj_set_style_pad_left(object, padding, selector);
    lv_obj_set_style_pad_right(object, padding, selector);
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
    lv_obj_move_to_index(object, -1);
}
