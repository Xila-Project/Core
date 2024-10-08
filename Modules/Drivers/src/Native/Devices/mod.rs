mod Drive_file;
mod SDL2;

use std::sync::Arc;

pub use Drive_file::*;
use File_system::Device_type;
use Graphics::Point_type;
use Task::Task_identifier_type;
pub use SDL2::*;

pub fn Mount_devices(
    Task: Task_identifier_type,
    Virtual_file_systems: &File_system::Virtual_file_system_type,
) -> Result<(), String> {
    const Resolution: Point_type = Point_type::New(800, 600);

    let (Screen_device, Pointer_device) =
        New_touchscreen(Resolution).expect("Error creating touchscreen");

    Virtual_file_systems
        .Mount_device(
            Task,
            "/Devices/Pointer",
            Device_type::New(Arc::new(Pointer_device)),
            false,
        )
        .expect("Error adding pointer device");

    Virtual_file_systems
        .Mount_device(
            Task,
            "/Devices/Screen",
            Device_type::New(Arc::new(Screen_device)),
            false,
        )
        .expect("Error adding screen device");

    Ok(())
}
