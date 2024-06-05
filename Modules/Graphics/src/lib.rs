#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Window;

use lvgl::{
    self,
    input_device::{pointer, InputDriver},
};

use Screen::Prelude::*;

pub struct Draw_buffer_type<const Buffer_size: usize>(lvgl::DrawBuffer<Buffer_size>);

impl<const Buffer_size: usize> Default for Draw_buffer_type<Buffer_size> {
    fn default() -> Self {
        Draw_buffer_type(lvgl::DrawBuffer::<Buffer_size>::default())
    }
}

pub struct Input_type(pointer::Pointer);

impl Input_type {
    pub fn New<Pointer_type: Input_traits>(
        Pointer: &Pointer_type,
        Display: &Display_type,
    ) -> Result<Self, ()> {
        let Binding_function = || {
            let (Position, Touch) = Pointer.Get_latest_input();
            let Input_data =
                pointer::PointerInputData::Touch((Position.X as i32, Position.Y as i32).into());

            let Input_data = match Touch {
                Touch_type::Pressed => Input_data.pressed(),
                Touch_type::Released => Input_data.released(),
            };

            //    if Changed {
            Input_data.once()
            //    } else {
            //       Input_data.and_continued()
            //   }
        };

        match pointer::Pointer::register(Binding_function, Display.Get_lvgl_display()) {
            Ok(Input) => Ok(Input_type(Input)),
            Err(_) => Err(()),
        }
    }
}

pub struct Display_type(lvgl::Display);

impl Display_type {
    pub fn New<Screen_type: Screen_traits<Buffer_size>, const Buffer_size: usize>(
        Screen: &mut Screen_type,
        Draw_buffer: lvgl::DrawBuffer<Buffer_size>,
    ) -> Result<Self, ()> {
        let Resolution = match Screen.Get_resolution() {
            Ok(Resolution) => Resolution,
            Err(_) => return Err(()),
        };

        let Binding_function = |Refresh: &lvgl::DisplayRefresh<Buffer_size>| {
            let Area = Area_type::New(
                Point_type::New(Refresh.area.x1 as i16, Refresh.area.y1 as i16),
                Point_type::New(
                    (Refresh.area.x2 - Refresh.area.x1) as i16,
                    (Refresh.area.y2 - Refresh.area.y1) as i16,
                ),
            );

            let mut Buffer = [Color_type {
                Red: 0,
                Green: 0,
                Blue: 0,
            }; Buffer_size];

            for i in 0..Buffer_size {
                let Color = Refresh.colors[i];

                let Color = Color_type {
                    Red: Color.r() << 3,
                    Green: Color.g() << 2,
                    Blue: Color.b() << 3,
                };

                Buffer[i] = Color;
            }

            let Refresh_area = Refresh_area_type::<Buffer_size> { Area, Buffer };

            Screen.Update(&Refresh_area);
        };

        let LVGL_display = match lvgl::Display::register(
            Draw_buffer,
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

#[cfg(test)]
mod tests {
    use super::*;
    use lvgl::Widget;
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    #[test]
    #[ignore]
    fn Test_SDL2() {
        const Horizontal_resolution: u32 = 800;
        const Vertical_resolution: u32 = 480;
        const Buffer_size: usize = (Horizontal_resolution * Vertical_resolution / 2) as usize;

        let Touchscreen = New_touchscreen(Point_type::New(
            Horizontal_resolution as i16,
            Vertical_resolution as i16,
        ));
        assert!(Touchscreen.is_ok());
        let (mut Screen, mut Pointer) = Touchscreen.unwrap();

        let Buffer = lvgl::DrawBuffer::<Buffer_size>::default();

        let Display = Display_type::New(&mut Screen, Buffer);
        assert!(Display.is_ok());
        let Display = Display.unwrap();

        let _Input = Input_type::New(&Pointer, &Display);
        assert!(_Input.is_ok());
        let mut _Input = _Input.unwrap();

        let Display_object = Display.Get_object();
        assert!(Display_object.is_ok());
        let mut Display_object = Display_object.unwrap();

        let _ = lvgl::widgets::Slider::create(&mut Display_object);

        let Calendar = lvgl::widgets::Calendar::create(&mut Display_object);
        assert!(Calendar.is_ok());
        let mut Calendar = Calendar.unwrap();

        let mut Style = lvgl::style::Style::default();
        Style.set_bg_color(lvgl::Color::from_rgb((255, 0, 0)));

        let _ = Calendar.add_style(lvgl::obj::Part::Any, &mut Style);
        let _ = Calendar.set_align(lvgl::Align::Center, 0, 0);

        loop {
            let Start = Instant::now();
            lvgl::task_handler();
            sleep(Duration::from_millis(5));
            lvgl::tick_inc(Instant::now().duration_since(Start));
            Pointer.Update();
        }
    }
}
