use std::mem::transmute;

use Screen::{Area_type, Point_type, Screen_traits};

use crate::{Color_type, Draw_buffer::Draw_buffer_type, Result_type};

pub struct Display_type {
    Display: lvgl::Display,
    #[allow(dead_code)]
    Screen: Box<dyn Screen_traits<Color_type>>,
}

unsafe impl Send for Display_type {}

unsafe impl Sync for Display_type {}

impl Display_type {
    pub fn New<const Buffer_size: usize>(
        mut Screen: Box<dyn Screen_traits<Color_type>>,
        Resolution: Point_type,
    ) -> Result_type<Self> {
        let Binding_function = |Refresh: &lvgl::DisplayRefresh<Buffer_size>| {
            let Area = Area_type::New(
                Point_type::New(Refresh.area.x1, Refresh.area.y1),
                Point_type::New(Refresh.area.x2, Refresh.area.y2),
            );

            let Buffer: &[Color_type; Buffer_size] = unsafe {
                transmute(
                    // Avoid copying the buffer, but the colors must be in the same format
                    &Refresh.colors,
                )
            };

            let _ = Screen.Update(Area, Buffer);
        };

        let Draw_buffer = Draw_buffer_type::<Buffer_size>::default();

        let LVGL_display = lvgl::Display::register(
            Draw_buffer.into(),
            Resolution.Get_x() as u32,
            Resolution.Get_y() as u32,
            Binding_function,
        )?;

        Ok(Self {
            Display: LVGL_display,
            Screen,
        })
    }

    pub fn Get_lvgl_display(&self) -> &lvgl::Display {
        &self.Display
    }

    pub fn Get_object(&self) -> Result_type<lvgl::Screen> {
        Ok(self.Display.get_scr_act()?)
    }
}
