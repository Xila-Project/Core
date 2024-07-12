use lvgl::input_device::{pointer, InputDriver};
use File_system::File_type;
use Screen::Prelude::Touch_type;

use crate::{Display::Display_type, Result_type};

pub struct Input_type {
    #[allow(dead_code)]
    Pointer: pointer::Pointer,
}

unsafe impl Send for Input_type {}

unsafe impl Sync for Input_type {}

impl Input_type {
    pub fn New(File: File_type, Display: &Display_type) -> Result_type<Self> {
        let Binding_closure = move || {
            let mut Buffer = [0u8; 5];

            let Size = File
                .Read(&mut Buffer)
                .expect("Error reading from input device");

            if Size != Buffer.len() {
                panic!("Invalid input data received from input device");
            }

            let X = u16::from_le_bytes([Buffer[0], Buffer[1]]);
            let Y = u16::from_le_bytes([Buffer[2], Buffer[3]]);

            let Touch = Touch_type::try_from(Buffer[4])
                .expect("Invalid touch type received from input device");

            let Input_data = pointer::PointerInputData::Touch((X as i32, Y as i32).into());

            let Input_data = match Touch {
                Touch_type::Pressed => Input_data.pressed(),
                Touch_type::Released => Input_data.released(),
            };

            Input_data.once()
        };

        Binding_closure();

        Ok(Self {
            Pointer: pointer::Pointer::register(Binding_closure, Display.Get_lvgl_display())?,
            // File: File,
        })
    }
}
