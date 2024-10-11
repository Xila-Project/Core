mod Drive_file;
mod SDL2;

pub use Drive_file::*;
use Graphics::Point_type;
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;
pub use SDL2::*;

pub fn Mount_devices(
    Task: Task_identifier_type,
    Virtual_file_systems: &Virtual_file_system_type,
) -> Result<(), String> {
    const Resolution: Point_type = Point_type::New(800, 600);

    let (Screen_device, Pointer_device) =
        New_touchscreen(Resolution).expect("Error creating touchscreen");

    Virtual_file_systems
        .Mount_device(Task, "/Devices/Pointer", Pointer_device, false)
        .expect("Error adding pointer device");

    Virtual_file_systems
        .Mount_device(Task, "/Devices/Screen", Screen_device, false)
        .expect("Error adding screen device");

    Ok(())
}
