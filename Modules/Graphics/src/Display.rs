use File_system::File_type;

use crate::{Draw_buffer::Draw_buffer_type, Point_type, Result_type, Screen_write_data_type};

pub struct Display_type {
    Display: lvgl::Display,
}

unsafe impl Send for Display_type {}

unsafe impl Sync for Display_type {}

impl Display_type {
    pub fn New<const Buffer_size: usize>(
        File: File_type,
        Resolution: Point_type,
    ) -> Result_type<Self> {
        let Binding_function = move |Refresh: &lvgl::DisplayRefresh<Buffer_size>| {
            let Buffer: &Screen_write_data_type<Buffer_size> = Refresh.as_ref();

            File.Write(Buffer.as_ref())
                .expect("Error writing to display");
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
        })
    }

    pub fn Get_lvgl_display(&self) -> &lvgl::Display {
        &self.Display
    }

    pub fn Get_object(&self) -> Result_type<lvgl::Screen> {
        Ok(self.Display.get_scr_act()?)
    }
}
