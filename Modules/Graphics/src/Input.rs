use lvgl::input_device::{pointer, InputDriver};
use Screen::Prelude::{Input_traits, Touch_type};

use crate::Display::Display_type;

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

            Input_data.once()
        };

        match pointer::Pointer::register(Binding_function, Display.Get_lvgl_display()) {
            Ok(Input) => Ok(Input_type(Input)),
            Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Draw_buffer_type;

    use super::*;
    use lvgl::Widget;
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };
    use Screen::{Drivers::SDL2::New_touchscreen, Prelude::Point_type};

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

        let Buffer = Draw_buffer_type::<Buffer_size>::default();

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
