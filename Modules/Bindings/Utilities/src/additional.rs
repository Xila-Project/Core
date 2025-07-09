use proc_macro2::TokenStream;
use quote::quote;

pub fn Get() -> TokenStream {
    quote! {
        pub unsafe fn object_delete(__pointer_table : &mut Pointer_table_type, __task: Task_identifier_type, object: u16) {

            let object = __pointer_table.remove(__task, object).unwrap();

            lv_obj_delete(object);
        }

        pub unsafe fn window_create() -> *mut lv_obj_t {
            Futures::block_on(
                graphics::get_instance().create_window()
            ).unwrap().into_raw()
        }

        pub unsafe fn window_pop_event(
            __environment: Environment_type,
            __pointer_table: &mut Pointer_table_type,
            window: *mut lv_obj_t,
            Code: *mut u32,
            Target: *mut u16
        ) {
            let mut window = graphics::Window_type::from_raw(window);

            if let Some(event) = window.pop_event() {

                *Code = event.get_code() as u32;

                *Target = __pointer_table
                    .get_wasm_pointer(event.get_target())
                    .unwrap();
            }

            core::mem::forget(window);
        }

        pub unsafe fn window_get_event_code(window: *mut lv_obj_t) -> u32 {
            let window = graphics::Window_type::from_raw(window);

            let code = if let Some(event) = window.peek_event() {
                event.get_code() as u32
            } else {
                graphics::Event_code_type::All as u32
            };

            core::mem::forget(window);

            code
        }

        pub unsafe fn window_get_event_target(__pointer_table: &mut Pointer_table_type, window: *mut lv_obj_t) -> u16 {
            let window = graphics::Window_type::from_raw(window);

            let target = if let Some(event) = window.peek_event() {
                event.get_target()
            } else {
                log::Warning!("No event available for the window");
                core::ptr::null_mut()
            };

            core::mem::forget(window);

            __pointer_table.get_wasm_pointer(target)
                .unwrap()
        }

        pub unsafe fn window_next_event(window: *mut lv_obj_t) {
            let mut window = graphics::Window_type::from_raw(window);

            window.pop_event();

            core::mem::forget(window);
        }

        pub unsafe fn buttonmatrix_set_map(__environment : Environment_type, __pointer_table : &mut Pointer_table_type, __task: Task_identifier_type, object: u16, map: *const *const i8) {

            let mut v : Vec<*const i8> = vec![];
            let mut i = 0;

            // Treat Map as a pointer to u32 values instead
            let map_as_u32 = map as *const u32;
            loop {
                let val = unsafe { *map_as_u32.add(i) };

                let val : *const i8 = convert_to_native_pointer(&__environment, val).unwrap();

                v.push(val);

                // Check if the converted pointer points to an empty string
                if *val == 0 {
                    break;
                }

                i += 1;

            }

            let object = __pointer_table.get_native_pointer(__task, object).unwrap();
            lv_buttonmatrix_set_map(object, v.as_ptr());

            core::mem::forget(v);   // ! : deallocate the vector to avoid memory leaks when the button matrix map is deleted
        }

        pub unsafe fn percentage(
            value: i32,
        ) -> i32 {
            lv_pct(value)
        }

    }
}
