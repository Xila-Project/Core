use lvgl::input_device::{pointer, InputDriver};
use File_system::File_type;

use crate::{Display::Display_type, Pointer_data_type, Result_type};

pub struct Input_type {
    #[allow(dead_code)]
    Pointer: pointer::Pointer,
}

unsafe impl Send for Input_type {}

unsafe impl Sync for Input_type {}

impl Input_type {
    pub fn New(File: File_type, Display: &Display_type) -> Result_type<Self> {
        let Binding_closure = move || {
            let mut Pointer_data = Pointer_data_type::default();

            File.Read(Pointer_data.as_mut())
                .expect("Error reading from input device");

            Pointer_data.into()
        };

        Binding_closure();

        Ok(Self {
            Pointer: pointer::Pointer::register(Binding_closure, Display.Get_lvgl_display())?,
            // File: File,
        })
    }
}
