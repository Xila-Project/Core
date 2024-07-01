#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{
    thread::{self, sleep},
    time::{Duration, Instant},
};

use Bindings::{File_system_bindings, Graphics_bindings, Task_bindings};
use File_system::{
    Drivers::Native::File_system_type,
    Prelude::{Path_type, Virtual_file_system_type},
};
use Graphics::{lvgl, Display_type, Draw_buffer_type, Input_type};
use Screen::{Drivers::SDL2::New_touchscreen, Prelude::Point_type};
use Virtual_machine::{Data_type, Instantiate_test_environment, WasmValue};

#[test]
fn Integration_test() {
    let Binary_buffer = include_bytes!(
        "../../../target/wasm32-unknown-unknown/release/File_system_bindings_WASM_test.wasm"
    );

    const Horizontal_resolution: u32 = 800;
    const Vertical_resolution: u32 = 480;
    const Buffer_size: usize = (Horizontal_resolution * Vertical_resolution / 2) as usize;

    let Touchscreen = New_touchscreen(Point_type::New(
        Horizontal_resolution as i16,
        Vertical_resolution as i16,
    ));
    assert!(Touchscreen.is_ok());
    let (mut Screen, mut Pointer) = Touchscreen.unwrap();

    let Buffer = Draw_buffer_type::<Buffer_size>::default();

    let Display = Display_type::New(&mut Screen, Buffer);
    assert!(Display.is_ok());
    let Display = Display.unwrap();

    let _Input = Input_type::New(&Pointer, &Display);
    assert!(_Input.is_ok());
    let mut _Input = _Input.unwrap();

    let Display_object = Display.Get_object();
    assert!(Display_object.is_ok());
    let mut Display_object = Display_object.unwrap();

    thread::spawn(|| {
        let (_Runtime, _Module, Instance) = Instantiate_test_environment(
            Binary_buffer,
            Graphics_bindings::New(),
            &Data_type::New(),
        );

        assert_eq!(
            Instance
                .Call_export_function("Test_graphics", &vec![])
                .unwrap(),
            WasmValue::I32(42)
        )
    });

    loop {
        let Start = Instant::now();
        lvgl::task_handler();
        sleep(Duration::from_millis(5));
        lvgl::tick_inc(Instant::now().duration_since(Start));
        Pointer.Update();
    }
}
