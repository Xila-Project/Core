mod SDL2;

use Graphics::Point_type;
pub use SDL2::*;

pub fn Mount_devices(
    Virtual_file_systems: &File_system::Virtual_file_system_type,
) -> Result<(), String> {
    const Resolution: Point_type = Point_type::New(800, 600);

    let (Screen_device, Pointer_device) =
        New_touchscreen(Resolution).expect("Error creating touchscreen");

    Virtual_file_systems
        .Add_device(&"/Devices/Pointer", Box::new(Pointer_device))
        .expect("Error adding pointer device");

    Virtual_file_systems
        .Add_device(&"/Devices/Screen", Box::new(Screen_device))
        .expect("Error adding screen device");

    Ok(())
}
