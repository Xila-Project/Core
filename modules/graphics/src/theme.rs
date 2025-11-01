use alloc::boxed::Box;

use crate::{Color, Display, fonts, lvgl, palette};

pub const PRIMARY_COLOR: Color = Color::new(0xFA, 0xFA, 0xFA);
pub const BACKGROUND_COLOR_PRIMARY: Color = Color::new(0x09, 0x09, 0x0b);
pub const BACKGROUND_COLOR_PRIMARY_MUTED: Color = Color::new(0x18, 0x18, 0x1b);
pub const BORDER_COLOR_PRIMARY: Color = Color::new(0x27, 0x27, 0x2a);
pub const SECONDARY_COLOR: Color = palette::get(palette::Hue::Red, palette::Tone::MAIN);
pub const IS_DARK: bool = true;

/// Rust representation of LVGL's `lv_theme_t` structure
///
/// This struct is C FFI compatible and must match the memory layout of the C struct.
#[derive(Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Theme {
    pub apply_cb: Option<lvgl::lv_theme_apply_cb_t>,
    /// Apply the current theme's style on top of this theme.
    pub parent: *mut Theme,
    pub user_data: *mut core::ffi::c_void,
    pub disp: *mut lvgl::lv_display_t,
    pub color_primary: lvgl::lv_color_t,
    pub color_secondary: lvgl::lv_color_t,
    pub font_small: *const lvgl::lv_font_t,
    pub font_normal: *const lvgl::lv_font_t,
    pub font_large: *const lvgl::lv_font_t,
    /// Any custom flag used by the theme
    pub flags: u32,
}

pub(crate) fn initialize(display: &Display) {
    update(display, PRIMARY_COLOR, SECONDARY_COLOR, IS_DARK);

    unsafe {
        let default_theme = lvgl::lv_theme_default_get() as *const Theme;

        let theme = Box::new(*default_theme);
        let theme = Box::into_raw(theme) as *mut lvgl::lv_theme_t;

        lvgl::lv_theme_set_parent(theme, default_theme as *mut lvgl::lv_theme_t);
        lvgl::lv_theme_set_apply_cb(theme, Some(theme_apply));
        lvgl::lv_display_set_theme(display.get_lvgl_display(), theme);
        lvgl::lv_obj_set_style_bg_color(
            display.get_object(),
            BACKGROUND_COLOR_PRIMARY.into_lvgl_color(),
            lvgl::LV_PART_MAIN,
        );
    }
}

pub(crate) fn update(
    display: &Display,
    primary_color: Color,
    secondary_color: Color,
    is_dark: bool,
) {
    unsafe {
        lvgl::lv_theme_default_init(
            display.get_lvgl_display(),
            primary_color.into_lvgl_color(),
            secondary_color.into_lvgl_color(),
            is_dark,
            fonts::get_font_medium(),
        );
    }
}

pub fn get_background_color_primary() -> Color {
    BACKGROUND_COLOR_PRIMARY
}

pub fn get_background_color_primary_muted() -> Color {
    BACKGROUND_COLOR_PRIMARY_MUTED
}

pub fn get_primary_color() -> Color {
    PRIMARY_COLOR
}

pub fn get_secondary_color() -> Color {
    SECONDARY_COLOR
}

pub fn get_border_color_primary() -> Color {
    BORDER_COLOR_PRIMARY
}

/// Apply the default style to the given object
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
pub unsafe fn apply_default_style(
    object: *mut lvgl::lv_obj_t,
    selector: lvgl::lv_style_selector_t,
) {
    unsafe {
        lvgl::lv_obj_set_style_bg_color(
            object,
            BACKGROUND_COLOR_PRIMARY.into_lvgl_color(),
            selector,
        );
        lvgl::lv_obj_set_style_border_color(
            object,
            BORDER_COLOR_PRIMARY.into_lvgl_color(),
            selector,
        );
    }
}

/// Theme apply callback
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
pub unsafe extern "C" fn theme_apply(_: *mut lvgl::lv_theme_t, object: *mut lvgl::lv_obj_t) {
    unsafe {
        let class = lvgl::lv_obj_get_class(object);
        let parent = lvgl::lv_obj_get_parent(object);

        if class == &lvgl::lv_button_class {
            lvgl::lv_obj_set_style_bg_color(
                object,
                PRIMARY_COLOR.into_lvgl_color(),
                lvgl::LV_PART_MAIN,
            );
            lvgl::lv_obj_set_style_text_color(
                object,
                BACKGROUND_COLOR_PRIMARY.into_lvgl_color(),
                lvgl::LV_PART_MAIN,
            );

            let tab_view = lvgl::lv_obj_get_parent(parent);

            if !tab_view.is_null()
                && lvgl::lv_obj_get_child(tab_view, 0) == parent
                && lvgl::lv_obj_check_type(tab_view, &lvgl::lv_tabview_class)
            {
                lvgl::lv_obj_set_style_bg_color(
                    object,
                    BACKGROUND_COLOR_PRIMARY.into_lvgl_color(),
                    lvgl::LV_PART_MAIN,
                );

                lvgl::lv_obj_set_style_text_color(
                    object,
                    PRIMARY_COLOR.into_lvgl_color(),
                    lvgl::LV_PART_MAIN,
                );
            }
        } else if class == &lvgl::lv_buttonmatrix_class {
            apply_default_style(object, lvgl::LV_PART_MAIN);
            apply_default_style(object, lvgl::LV_PART_ITEMS);
            lvgl::lv_obj_set_style_text_color(
                object,
                PRIMARY_COLOR.into_lvgl_color(),
                lvgl::LV_PART_MAIN,
            );

            lvgl::lv_obj_set_style_border_width(object, 2, lvgl::LV_PART_ITEMS);
            lvgl::lv_obj_set_style_text_color(
                object,
                PRIMARY_COLOR.into_lvgl_color(),
                lvgl::LV_PART_ITEMS,
            );
        } else {
            apply_default_style(object, lvgl::LV_PART_MAIN);
        }
    }
}
