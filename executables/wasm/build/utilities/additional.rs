use proc_macro2::TokenStream;
use quote::quote;

pub fn get() -> TokenStream {
    quote! {
        fn window_create() -> *mut lv_obj_t;
        fn window_pop_event(window: *mut lv_obj_t, code: *mut u32, target: *mut *mut lv_obj_t);
        fn window_get_event_code(window: *mut lv_obj_t) -> u32;
        fn window_get_event_target(window: *mut lv_obj_t) -> *mut lv_obj_t;
        fn window_next_event(window: *mut lv_obj_t);
        fn window_set_icon(
            window: *mut lv_obj_t,
            icon: *mut lv_obj_t,
            recolor: lv_color_t,
            zoom: lv_pct,
        );
    }
}
