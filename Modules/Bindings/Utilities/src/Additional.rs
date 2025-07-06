use proc_macro2::TokenStream;
use quote::quote;

pub fn Get() -> TokenStream {
    quote! {
        pub unsafe fn Object_delete(__Pointer_table : &mut Pointer_table_type, __Task: Task_identifier_type, Object: u16) {

            let Object = __Pointer_table.Remove(__Task, Object).unwrap();

            lv_obj_delete(Object);
        }

        pub unsafe fn Window_create() -> *mut lv_obj_t {
            Futures::block_on(
                Graphics::Get_instance().Create_window()
            ).unwrap().Into_raw()
        }

        pub unsafe fn Window_pop_event(
            __Environment: Environment_type,
            __Pointer_table: &mut Pointer_table_type,
            Window: *mut lv_obj_t,
            Code: *mut u32,
            Target: *mut u16
        ) {
            let mut Window = Graphics::Window_type::From_raw(Window);

            if let Some(Event) = Window.Pop_event() {

                *Code = Event.Get_code() as u32;

                *Target = __Pointer_table
                    .Get_wasm_pointer(Event.Get_target())
                    .unwrap();
            }

            core::mem::forget(Window);
        }

        pub unsafe fn Window_get_event_code(Window: *mut lv_obj_t) -> u32 {
            let Window = Graphics::Window_type::From_raw(Window);

            let Code = if let Some(Event) = Window.Peek_event() {
                Event.Get_code() as u32
            } else {
                Graphics::Event_code_type::All as u32
            };

            core::mem::forget(Window);

            Code
        }

        pub unsafe fn Window_get_event_target(__Pointer_table: &mut Pointer_table_type, Window: *mut lv_obj_t) -> u16 {
            let Window = Graphics::Window_type::From_raw(Window);

            let Target = if let Some(Event) = Window.Peek_event() {
                Event.Get_target()
            } else {
                Log::Warning!("No event available for the window");
                core::ptr::null_mut()
            };

            core::mem::forget(Window);

            __Pointer_table.Get_wasm_pointer(Target)
                .unwrap()
        }

        pub unsafe fn Window_next_event(Window: *mut lv_obj_t) {
            let mut Window = Graphics::Window_type::From_raw(Window);

            Window.Pop_event();

            core::mem::forget(Window);
        }

        pub unsafe fn Buttonmatrix_set_map(__Environment : Environment_type, __Pointer_table : &mut Pointer_table_type, __Task: Task_identifier_type,  Object: u16, Map: *const *const i8) {

            let mut v : Vec<*const i8> = vec![];
            let mut i = 0;

            // Treat Map as a pointer to u32 values instead
            let map_as_u32 = Map as *const u32;
            loop {
                let val = unsafe { *map_as_u32.add(i) };

                let val : *const i8 = Convert_to_native_pointer(&__Environment, val).unwrap();

                v.push(val);

                // Check if the converted pointer points to an empty string
                if *val == 0 {
                    break;
                }

                i += 1;

            }

            let Object = __Pointer_table.Get_native_pointer(__Task, Object).unwrap();
            lv_buttonmatrix_set_map(Object, v.as_ptr());

            core::mem::forget(v);   // ! : deallocate the vector to avoid memory leaks when the button matrix map is deleted
        }

        pub unsafe fn Percentage(
            Value: i32,
        ) -> i32 {
            lv_pct(Value)
        }

    }
}
