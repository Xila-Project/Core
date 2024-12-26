use proc_macro2::TokenStream;
use quote::quote;

pub fn Get() -> TokenStream {
    quote! {
        pub unsafe fn Object_delete(__Pointer_table : &mut Pointer_table_type, __Task: Task_identifier_type, Object: u16) {

            let Object = __Pointer_table.Remove(__Task, Object).unwrap();

            lv_obj_delete(Object);
        }

        pub unsafe fn Window_create() -> *mut lv_obj_t {
            Graphics::Get_instance().Create_window().unwrap().Into_raw()

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

        pub unsafe fn Window_get_event_target(Window: *mut lv_obj_t) -> *mut lv_obj_t {
            let Window = Graphics::Window_type::From_raw(Window);

            let Target = if let Some(Event) = Window.Peek_event() {
                Event.Get_target()
            } else {
                core::ptr::null_mut()
            };

            core::mem::forget(Window);

            Target
        }

        pub unsafe fn Window_next_event(Window: *mut lv_obj_t) {
            let mut Window = Graphics::Window_type::From_raw(Window);

            Window.Pop_event();

            core::mem::forget(Window);
        }
    }
}
