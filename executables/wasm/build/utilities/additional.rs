use proc_macro2::TokenStream;
use quote::quote;

pub fn get() -> TokenStream {
    quote! {
        pub fn object_delete(__translation_map : &mut TranslationMap, __task: TaskIdentifier, object: u16);

        pub fn window_create() -> *mut lv_obj_t;

        pub fn window_pop_event(
            __environment: Environment,
            __translation_map: &mut TranslationMap,
            window: *mut lv_obj_t,
            code: *mut u32,
            target: *mut u16
        );

        pub fn window_get_event_code(window: *mut lv_obj_t) -> u32;

        pub fn window_get_event_target(__translation_map: &mut TranslationMap, window: *mut lv_obj_t) -> u16;

        pub fn window_next_event(window: *mut lv_obj_t);

        pub fn window_set_icon(
            __environment : Environment,
            __translation_map : &mut TranslationMap,
            __task: TaskIdentifier,
            window: *mut lv_obj_t,
            icon_string: *const core::ffi::c_char,
            icon_color: lv_color_t
        );

        pub fn buttonmatrix_set_map(
            __environment : Environment,
            __translation_map : &mut TranslationMap,
            __task: TaskIdentifier,
            object: u16,
            map: *const *const i8
        );

        pub fn percentage(value: i32) -> i32;
    }
}
