use std::usize;

use lvgl;
use sdl2::sys::Screen;

use crate::Screen::Prelude::*;

pub struct Display_type {
    Display: lvgl::Display,
}

impl Display_type {
    pub fn New<S, const Buffer_size: usize>(Screen: &mut S) -> Result<Self, ()>
    where
        S: Screen_traits<Buffer_size>,
    {
        let Resolution = match Screen.Get_resolution() {
            Ok(Resolution) => Resolution,
            Err(_) => return Err(()),
        };

        let Binding_function = |Refresh: &lvgl::DisplayRefresh<Buffer_size>| {

            print!("Binding function called.\n");

            let Area = Area_type {
                Position: Coordinates_type {
                    X: Refresh.area.x1,
                    Y: Refresh.area.y1,
                },
                Size: Coordinates_type {
                    X: (Refresh.area.x2 - Refresh.area.x1) as i16,
                    Y: (Refresh.area.y2 - Refresh.area.y1) as i16,
                },
            };

            let mut Buffer = [Color_type {
                Red: 0,
                Green: 0,
                Blue: 0,
            }; Buffer_size];

            for i in 0..Buffer_size {
                let Color = Refresh.colors[i];
                let Color = Color_type {
                    Red: Color.r(),
                    Green: Color.g(),
                    Blue: Color.b(),
                };

                Buffer[i] = Color;
            }

            let mut Refresh_area = Refresh_area_type::<Buffer_size> { Area, Buffer };

            Screen.Update(&Refresh_area);
        };

        let Buffer = lvgl::DrawBuffer::<Buffer_size>::default();

        let LVGL_display = match lvgl::Display::register(
            Buffer,
            Resolution.X as u32,
            Resolution.Y as u32,
            Binding_function,
        ) {
            Ok(Display) => Display,
            Err(_) => return Err(()),
        };

        Ok(Self {
            Display: LVGL_display,
        })
    }

    pub fn Get_object(&self) -> Result<lvgl::Obj, ()> {
        match self.Display.get_scr_act() {
            Ok(Object) => Ok(Object),
            Err(_) => Err(()),
        }
    }
}
