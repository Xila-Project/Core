#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use File_system::{File_type, Mode_type, Path_type};

const Pointer_device_path: &Path_type =
    unsafe { Path_type::New_unchecked_constant("/Device/Pointer") };

const Screen_device_path: &Path_type =
    unsafe { Path_type::New_unchecked_constant("/Device/Screen") };

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn main() {
    use Drivers::Native::New_touchscreen;
    use Graphics::{
        lvgl::{self, Widget},
        Get_recommended_buffer_size, Point_type,
    };

    const Resolution: Point_type = Point_type::New(800, 480);

    const Buffer_size: usize = Get_recommended_buffer_size(&Resolution);

    let (S, Pointer) =
        New_touchscreen::<Buffer_size>(Resolution).expect("Error creating touchscreen");

    Users::Initialize().expect("Error initializing users manager");

    let Task_instance = Task::Initialize().expect("Error initializing task manager");

    let Virtual_file_system = File_system::Initialize().expect("Error initializing file system");

    Virtual_file_system
        .Add_device(&Pointer_device_path, Box::new(Pointer))
        .expect("Error adding pointer device");

    Virtual_file_system
        .Add_device(&Screen_device_path, Box::new(S))
        .expect("Error adding screen device");

    let Task = Task_instance
        .Get_current_task_identifier()
        .expect("Failed to get current task identifier");

    let Pointer_file = File_type::Open(
        Virtual_file_system,
        Pointer_device_path,
        Mode_type::Read_only().into(),
        Task,
    )
    .expect("Error opening pointer file");

    let Screen_file = File_type::Open(
        Virtual_file_system,
        Screen_device_path,
        Mode_type::Read_write().into(),
        Task,
    )
    .expect("Error opening screen file");

    Graphics::Initialize().expect("Error initializing manager");

    let Graphics_manager = Graphics::Get_instance().expect("Error getting manager");

    let Display = Graphics_manager
        .Create_display::<Buffer_size>(Screen_file, Pointer_file)
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
