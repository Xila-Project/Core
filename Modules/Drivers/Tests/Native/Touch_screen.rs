#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use Drivers::Native::New_touchscreen;
use File_system::Device_trait;
use Screen::{Area_type, Color_ARGB8888_type, Error_type, Point_type, Screen_traits};

#[test]
#[cfg(target_os = "linux")]
fn Test_touchscreen() {
    const Horizontal_resolution: u32 = 800;
    const Vertical_resolution: u32 = 480;

    let Touchscreen = New_touchscreen(Point_type::New(
        Horizontal_resolution as i16,
        Vertical_resolution as i16,
    ));

    assert!(Touchscreen.is_ok());

    let (mut Screen, Pointer_device_type) =
        Touchscreen.expect("Touchscreen initialization failed.");

    let mut Buffer = [0; 5];

    assert_eq!(Pointer_device_type.Read(&mut Buffer), Ok(5));

    Screen
        .Update(
            Area_type::New(Point_type::New(0, 0), Point_type::New(9, 9)),
            &[Color_ARGB8888_type::New(255, 255, 255, 255); 100],
        )
        .expect("Screen update failed.");

    assert_eq!(
        Screen.Update(
            Area_type::New(Point_type::New(0, 0), Point_type::New(10, 9),),
            &[Color_ARGB8888_type::New(255, 255, 255, 255); 100],
        ),
        Err(Error_type::Invalid_dimension)
    );

    assert_eq!(Screen.Get_resolution().unwrap(), Point_type::New(800, 480));

    unsafe {
        sdl2::sys::SDL_Quit(); // Force SDL2 to quit to avoid conflicts with other tests.
    }
}
