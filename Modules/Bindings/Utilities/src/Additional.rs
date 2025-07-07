use proc_macro2::TokenStream;
use quote::quote;

pub fn Get() -> TokenStream {
    quote! {
        pub unsafe fn object_delete(__pointer_table : &mut Pointer_table_type, __task: Task_identifier_type, Object: u16) {

            let object = __pointer_table.Remove(__task, Object).unwrap();

            lv_obj_delete(object);
        }

        pub unsafe fn window_create() -> *mut lv_obj_t {
            Futures::block_on(
                Graphics::Get_instance().Create_window()
            ).unwrap().Into_raw()
        }

        pub unsafe fn window_pop_event(
            __environment: Environment_type,
            __pointer_table: &mut Pointer_table_type,
            window: *mut lv_obj_t,
            Code: *mut u32,
            Target: *mut u16
        ) {
            let mut window = Graphics::Window_type::From_raw(window);

            if let Some(event) = window.Pop_event() {

                *Code = event.Get_code() as u32;

                *Target = __pointer_table
                    .Get_wasm_pointer(event.Get_target())
                    .unwrap();
            }

            core::mem::forget(window);
        }

        pub unsafe fn window_get_event_code(window: *mut lv_obj_t) -> u32 {
            let window = Graphics::Window_type::From_raw(window);

            let code = if let Some(event) = window.Peek_event() {
                event.Get_code() as u32
            } else {
                Graphics::Event_code_type::All as u32
            };

            core::mem::forget(window);

            code
        }

        pub unsafe fn window_get_event_target(__pointer_table: &mut Pointer_table_type, window: *mut lv_obj_t) -> u16 {
            let window = Graphics::Window_type::From_raw(window);

            let target = if let Some(event) = window.Peek_event() {
                event.Get_target()
            } else {
                Log::Warning!("No event available for the window");
                core::ptr::null_mut()
            };

            core::mem::forget(window);

            __pointer_table.Get_wasm_pointer(target)
                .unwrap()
        }

        pub unsafe fn window_next_event(window: *mut lv_obj_t) {
            let mut window = Graphics::Window_type::From_raw(window);

            window.Pop_event();

            core::mem::forget(window);
        }

        pub unsafe fn buttonmatrix_set_map(__environment : Environment_type, __pointer_table : &mut Pointer_table_type, __task: Task_identifier_type,  Object: u16, Map: *const *const i8) {

            let mut v : Vec<*const i8> = vec![];
            let mut i = 0;

            // Treat Map as a pointer to u32 values instead
            let map_as_u32 = Map as *const u32;
            loop {
                let val = unsafe { *map_as_u32.add(i) };

                let val : *const i8 = Convert_to_native_pointer(&__environment, val).unwrap();

                v.push(val);

                // Check if the converted pointer points to an empty string
                if *val == 0 {
                    break;
                }

                i += 1;

            }

            let object = __pointer_table.Get_native_pointer(__task, Object).unwrap();
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
