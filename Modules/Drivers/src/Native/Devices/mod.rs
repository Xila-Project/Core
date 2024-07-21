mod SDL2;

use File_system::Path_type;
use Graphics::{Get_recommended_buffer_size, Point_type};
pub use SDL2::*;

const Pointer_device_path: &Path_type =
    unsafe { Path_type::New_unchecked_constant("/Devices/Pointer") };

const Screen_device_path: &Path_type =
    unsafe { Path_type::New_unchecked_constant("/Devices/Screen") };

pub fn Mount_devices(
    Virtual_file_systems: &File_system::Virtual_file_system_type,
) -> Result<(), String> {
    const Resolution: Point_type = Point_type::New(800, 600);

    const Buffer_size: usize = Get_recommended_buffer_size(&Resolution);

    let (Screen_device, Pointer_device) =
        New_touchscreen::<Buffer_size>(Resolution).expect("Error creating touchscreen");

    Virtual_file_systems
        .Add_device(&Pointer_device_path, Box::new(Pointer_device))
        .expect("Error adding pointer device");

    Virtual_file_systems
        .Add_device(&Screen_device_path, Box::new(Screen_device))
        .expect("Error adding screen device");

    Ok(())
}
