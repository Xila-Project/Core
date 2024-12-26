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
    Object: *mut lv_obj_t,
    Padding: i32,
    Selector: lv_style_selector_t,
) {
    lv_obj_set_style_pad_top(Object, Padding, Selector);
    lv_obj_set_style_pad_bottom(Object, Padding, Selector);
    lv_obj_set_style_pad_left(Object, Padding, Selector);
    lv_obj_set_style_pad_right(Object, Padding, Selector);
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
pub unsafe fn lv_obj_move_foreground(Object: *mut lv_obj_t) {
    lv_obj_move_to_index(Object, -1);
}
