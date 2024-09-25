#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

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
    use Graphics::{lvgl, Get_recommended_buffer_size, Point_type};
    use Time::Duration_type;

    Users::Initialize().expect("Error initializing users manager");

    let Task_instance = Task::Initialize().expect("Error initializing task manager");

    Time::Initialize(Box::new(Drivers::Native::Time_driver_type::New()))
        .expect("Error initializing time manager");

    const Resolution: Point_type = Point_type::New(800, 480);

    const Buffer_size: usize = Get_recommended_buffer_size(&Resolution);

    let (S, Pointer) = New_touchscreen(Resolution).expect("Error creating touchscreen");

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
        Mode_type::Read_only.into(),
        Task,
    )
    .expect("Error opening pointer file");

    let Screen_file = File_type::Open(
        Virtual_file_system,
        Screen_device_path,
        Mode_type::Read_write.into(),
        Task,
    )
    .expect("Error opening screen file");

    Graphics::Initialize().expect("Error initializing manager");

    let Graphics_manager = Graphics::Get_instance().expect("Error getting manager");

    let Display = Graphics_manager
        .Create_display::<Buffer_size>(Screen_file, Pointer_file, false)
        .expect("Error adding screen");

    let Screen_object = Display.Get_object();

    let _Calendar = unsafe { lvgl::lv_calendar_create(Screen_object) };

    loop {
        Graphics::Manager_type::Loop();

        Task::Manager_type::Sleep(Duration_type::from_millis(1000));
    }
}
