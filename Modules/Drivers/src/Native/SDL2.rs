use Screen::{
    Area_type, Color_ARGB8888_type, Color_RGB565_type, Error_type, Point_type, Result_type,
    Screen_traits, Touch_type,
};

use sdl2::{
    event, mouse, pixels,
    render::Canvas,
    video::{self, WindowBuildError},
    EventPump, IntegerOrSdlError,
};
use File_system::Device_trait;

use std::{mem::size_of, process::exit, sync::RwLock};

fn From_string_error(Value: String) -> Error_type {
    Error_type::Unknown(Value)
}

fn From_window_build_error(Error: WindowBuildError) -> Error_type {
    match Error {
        WindowBuildError::SdlError(Error) => From_string_error(Error),
        WindowBuildError::HeightOverflows(_) | WindowBuildError::WidthOverflows(_) => {
            Error_type::Invalid_dimension
        }
        WindowBuildError::InvalidTitle(_) => Error_type::Unknown("Invalid title.".to_string()),
    }
}

fn From_integer_or_sdl_error(Error: IntegerOrSdlError) -> Error_type {
    Error_type::Unknown(Error.to_string())
}

pub struct Screen_type(Canvas<video::Window>);

impl Screen_type {
    pub fn New(Window: video::Window) -> Result_type<Self> {
        let mut Canvas = Window
            .into_canvas()
            .build()
            .map_err(From_integer_or_sdl_error)?;

        Canvas.clear();
        Canvas.present();

        Ok(Self(Canvas))
    }

    pub fn Get_resolution(&self) -> Result_type<Point_type> {
        self.0
            .output_size()
            .map(|(Width, Height)| Point_type::New(Width as i16, Height as i16))
            .map_err(From_string_error)
    }
}

impl Screen_traits<Color_RGB565_type> for Screen_type {
    fn Update(&mut self, Area: Area_type, Buffer: &[Color_RGB565_type]) -> Result_type<()> {
        let mut Buffer_iterator = Buffer.iter();

        let Point_1 = Area.Get_point_1();
        let Point_2 = Area.Get_point_2();

        for Y in Point_1.Get_y() as i32..=Point_2.Get_y() as i32 {
            for X in Point_1.Get_x() as i32..=Point_2.Get_x() as i32 {
                let Color = Buffer_iterator
                    .next()
                    .ok_or(Error_type::Invalid_dimension)?;

                let Color: Color_ARGB8888_type = (*Color).into();

                self.0.set_draw_color(pixels::Color::RGB(
                    Color.Get_red(),
                    Color.Get_green(),
                    Color.Get_blue(),
                ));

                let _ = self.0.draw_point(sdl2::rect::Point::new(X, Y));
            }
        }
        self.0.present();

        Ok(())
    }
}

impl Screen_traits<Color_ARGB8888_type> for Screen_type {
    fn Update(&mut self, Area: Area_type, Buffer: &[Color_ARGB8888_type]) -> Result_type<()> {
        let mut Buffer_iterator = Buffer.iter();

        let Point_1 = Area.Get_point_1();
        let Point_2 = Area.Get_point_2();

        for Y in Point_1.Get_y() as i32..=Point_2.Get_y() as i32 {
            for X in Point_1.Get_x() as i32..=Point_2.Get_x() as i32 {
                let Color = Buffer_iterator
                    .next()
                    .ok_or(Error_type::Invalid_dimension)?;

                self.0.set_draw_color(pixels::Color::RGB(
                    Color.Get_red(),
                    Color.Get_green(),
                    Color.Get_blue(),
                ));

                let _ = self.0.draw_point(sdl2::rect::Point::new(X, Y));
            }
        }
        self.0.present();

        Ok(())
    }
}

pub struct Pointer_device_type {
    Window_identifier: u32,
    Event_pump: RwLock<EventPump>,
    Last_input: RwLock<(Point_type, Touch_type)>,
}

unsafe impl Send for Pointer_device_type {}

unsafe impl Sync for Pointer_device_type {}

impl Pointer_device_type {
    pub fn New(Window_identifier: u32, Event_pump: EventPump) -> Self {
        Self {
            Window_identifier,
            Event_pump: RwLock::new(Event_pump),
            Last_input: RwLock::new((Point_type::New(0, 0), Touch_type::Released)),
        }
    }

    pub fn Update(&self) -> Result_type<()> {
        let mut Last_input = self.Last_input.write()?;

        for Event in self.Event_pump.write()?.poll_iter() {
            match Event {
                event::Event::Quit { .. } => exit(0),
                event::Event::MouseButtonDown {
                    timestamp: _,
                    window_id,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x,
                    y,
                } => {
                    if (window_id == self.Window_identifier)
                        && (mouse_btn == mouse::MouseButton::Left)
                    {
                        Last_input.0 = Last_input.0.Set(x as i16, y as i16);
                        Last_input.0 = Last_input.0.Set(x as i16, y as i16);

                        Last_input.1 = Touch_type::Pressed;
                    }
                }
                event::Event::MouseButtonUp {
                    timestamp: _,
                    window_id,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    ..
                } => {
                    if (window_id == self.Window_identifier)
                        && (mouse_btn == mouse::MouseButton::Left)
                    {
                        Last_input.1 = Touch_type::Released;
                    }
                }
                event::Event::MouseMotion {
                    timestamp: _,
                    window_id,
                    which: _,
                    mousestate,
                    x,
                    y,
                    ..
                } => {
                    if (window_id == self.Window_identifier) && (mousestate.left()) {
                        Last_input.0 = Last_input.0.Set(x as i16, y as i16);
                    }
                }
                _ => {}
            };
        }

        Ok(())
    }
}

impl Device_trait for Pointer_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<usize> {
        if self.Update().is_err() {
            return Err(File_system::Error_type::Internal_error);
        }

        let Last_input = self.Last_input.read()?;

        Buffer[0..2].copy_from_slice(&Last_input.0.Get_x().to_le_bytes());
        Buffer[2..4].copy_from_slice(&Last_input.0.Get_y().to_le_bytes());
        Buffer[4] = Last_input.1.into();

        Ok(5)
    }

    fn Write(&self, _: &[u8]) -> File_system::Result_type<usize> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Get_size(&self) -> File_system::Result_type<usize> {
        Ok(size_of::<Self>())
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<usize> {
        Err(File_system::Error_type::Unsupported_operation)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}

pub fn New_touchscreen(Size: Point_type) -> Result_type<(Screen_type, Pointer_device_type)> {
    let Context = sdl2::init().map_err(From_string_error)?;

    let Video_subsystem = Context.video().map_err(From_string_error)?;

    let Window = Video_subsystem
        .window("Xila", Size.Get_x() as u32, Size.Get_y() as u32)
        .position_centered()
        .build()
        .map_err(From_window_build_error)?;

    let Event_pump = Context.event_pump().map_err(From_string_error)?;

    let Pointer = Pointer_device_type::New(Window.id(), Event_pump);

    let Screen = Screen_type::New(Window)?;

    Ok((Screen, Pointer))
}
