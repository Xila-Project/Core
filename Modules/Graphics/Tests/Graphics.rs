#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use lvgl::Widget;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use File_system::{File_type, Mode_type, Path_type};
use Screen::{Drivers::SDL2::New_touchscreen, Prelude::Point_type};

const Pointer_device_path: &Path_type =
    unsafe { Path_type::New_unchecked_constant("/Device/Pointer") };

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn main() {
    lvgl::init();

    const Horizontal_resolution: u32 = 800;
    const Vertical_resolution: u32 = 480;
    const Buffer_size: usize = (Horizontal_resolution * Vertical_resolution / 2) as usize;

    let (S, Pointer) = New_touchscreen(Point_type::New(
        Horizontal_resolution as i16,
        Vertical_resolution as i16,
    ))
    .expect("Error creating touchscreen");

    let S = Box::new(S);

    Users::Initialize().expect("Error initializing users manager");

    Task::Initialize().expect("Error initializing task manager");

    let Virtual_file_system = File_system::Initialize().expect("Error initializing file system");

    Virtual_file_system
        .Add_device(&Pointer_device_path, Box::new(Pointer))
        .expect("Error adding pointer device");

    let Pointer_file = File_type::Open(
        Virtual_file_system,
        Pointer_device_path,
        Mode_type::Read_only().into(),
    )
    .expect("Error opening pointer file");

    Graphics::Initialize().expect("Error initializing manager");

    let Graphics_manager = Graphics::Get_instance().expect("Error getting manager");

    let Resolution = S.Get_resolution().expect("Error getting resolution");

    let Display = Graphics_manager
        .Create_display::<Buffer_size>(S, Resolution, Pointer_file)
        .expect("Error adding screen");

    let mut S = Display.Get_object().expect("Error getting screen");

    let _ = lvgl::widgets::Slider::create(&mut S);

    let Calendar = lvgl::widgets::Calendar::create(&mut S);
    assert!(Calendar.is_ok());
    let mut Calendar = Calendar.unwrap();

    let mut Style = lvgl::style::Style::default();
    Style.set_bg_color(lvgl::Color::from_rgb((255, 0, 0)));

    Calendar.add_style(lvgl::obj::Part::Any, &mut Style);
    Calendar.set_align(lvgl::Align::Center, 0, 0);

    loop {
        let Start = Instant::now();
        lvgl::task_handler();
        sleep(Duration::from_millis(5));
        lvgl::tick_inc(Instant::now().duration_since(Start));
    }
}
