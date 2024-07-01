use Screen::Prelude::{Area_type, Color_type, Point_type, Refresh_area_type, Screen_traits};

use crate::Draw_buffer_type;

pub struct Display_type(lvgl::Display);

impl Display_type {
    pub fn New<Screen_type: Screen_traits<Buffer_size>, const Buffer_size: usize>(
        Screen: &mut Screen_type,
        Draw_buffer: Draw_buffer_type<Buffer_size>,
    ) -> Result<Self, ()> {
        let Resolution = match Screen.Get_resolution() {
            Ok(Resolution) => Resolution,
            Err(_) => return Err(()),
        };

        let Binding_function = |Refresh: &lvgl::DisplayRefresh<Buffer_size>| {
            let Area = Area_type::New(
                Point_type::New(Refresh.area.x1, Refresh.area.y1),
                Point_type::New(
                    Refresh.area.x2 - Refresh.area.x1,
                    Refresh.area.y2 - Refresh.area.y1,
                ),
            );

            let mut Buffer = [Color_type {
                Red: 0,
                Green: 0,
                Blue: 0,
            }; Buffer_size];

            for (Destination, Source) in Buffer.iter_mut().zip(Refresh.colors.iter()) {
                *Destination = Color_type {
                    Red: Source.r() << 3,
                    Green: Source.g() << 2,
                    Blue: Source.b() << 3,
                };
            }

            let Refresh_area = Refresh_area_type::<Buffer_size> { Area, Buffer };

            Screen.Update(&Refresh_area);
        };

        let LVGL_display = match lvgl::Display::register(
            Draw_buffer.into(),
            Resolution.X as u32,
            Resolution.Y as u32,
            Binding_function,
        ) {
            Ok(Display) => Display,
            Err(_) => return Err(()),
        };

        Ok(Display_type(LVGL_display))
    }

    pub fn Get_lvgl_display(&self) -> &lvgl::Display {
        &self.0
    }

    pub fn Get_object(&self) -> Result<lvgl::Screen, ()> {
        match self.0.get_scr_act() {
            Ok(Object) => Ok(Object),
            Err(_) => Err(()),
        }
    }
}
