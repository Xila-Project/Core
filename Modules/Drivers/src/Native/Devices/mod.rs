mod Drive_file;
pub mod Window_screen;

pub use Drive_file::*;
use Graphics::Point_type;
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;
pub use SDL2::*;

pub fn Mount_devices(
    Task: Task_identifier_type,
    Virtual_file_system: &Virtual_file_system_type,
) -> Result<(), String> {
    const Resolution: Point_type = Point_type::New(800, 600);

    let (Screen, Pointer) = Window_screen::New(Resolution)?;

    Virtual_file_system
        .Mount_device(Task, "/Devices/Screen", Screen)
        .map_err(|Error| format!("Error adding screen device: {:?}", Error))?;

    Virtual_file_system
        .Mount_device(Task, "/Devices/Pointer", Pointer)
        .map_err(|Error| format!("Error adding pointer device: {:?}", Error))?;


    Ok(())
}
