use crate::{
    Generics,
    Prelude::{Error_type, Result_type},
};

use sdl2::{
    event, mouse, pixels,
    render::Canvas,
    video::{self, WindowBuildError},
    EventPump, IntegerOrSdlError,
};

use std::{
    cell::RefCell,
    process::exit,
    sync::{Arc, RwLock},
};

impl From<String> for Error_type {
    fn from(Value: String) -> Self {
        Self::Unknown(Value)
    }
}

impl From<WindowBuildError> for Error_type {
    fn from(Error: WindowBuildError) -> Self {
        match Error {
            WindowBuildError::SdlError(Error) => Error.into(),
            WindowBuildError::HeightOverflows(_) | WindowBuildError::WidthOverflows(_) => {
                Error_type::Invalid_dimension
            }
            WindowBuildError::InvalidTitle(_) => Error_type::Unknown("Invalid title.".to_string()),
        }
    }
}

impl From<IntegerOrSdlError> for Error_type {
    fn from(Error: IntegerOrSdlError) -> Self {
        Error_type::Unknown(Error.to_string())
    }
}

pub struct Screen_type(Canvas<video::Window>);

impl Screen_type {
    pub fn New(Window: video::Window) -> Result_type<Self> {
        let mut Canvas = Window.into_canvas().build()?;

        Canvas.clear();
        Canvas.present();

        Ok(Self(Canvas))
    }
}

impl<const Buffer_size: usize> Generics::Screen_traits<Buffer_size> for Screen_type {
    fn Update(&mut self, Refresh_area: &Generics::Refresh_area_type<Buffer_size>) {
        let mut Buffer_iterator = Refresh_area.Buffer.iter();

        for Y in 0..=Refresh_area.Area.Size.Y {
            for X in 0..=Refresh_area.Area.Size.X {
                let Color = Buffer_iterator.next().unwrap();

                self.0
                    .set_draw_color(pixels::Color::RGB(Color.Red, Color.Green, Color.Blue));
                let _ = self.0.draw_point(sdl2::rect::Point::new(
                    (X + Refresh_area.Area.Position.X) as i32,
                    (Y + Refresh_area.Area.Position.Y) as i32,
                ));
            }
        }
        self.0.present();
    }

    fn Get_resolution(&self) -> Result_type<Generics::Point_type> {
        Ok(self
            .0
            .output_size()
            .map(|(Width, Height)| Generics::Point_type::New(Width as i16, Height as i16))?)
    }
}

pub struct Pointer_type {
    Window_identifier: u32,
    Event_pump: RefCell<EventPump>,
    Last_input: Arc<RwLock<(Generics::Point_type, Generics::Touch_type)>>,
}

impl Pointer_type {
    pub fn New(Window_identifier: u32, Event_pump: EventPump) -> Self {
        Self {
            Window_identifier,
            Event_pump: RefCell::new(Event_pump),
            Last_input: Arc::new(RwLock::new((
                Generics::Point_type::New(0, 0),
                Generics::Touch_type::Released,
            ))),
        }
    }

    pub fn Update(&mut self) {
        for Event in self.Event_pump.borrow_mut().poll_iter() {
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
                        let mut Last_input = self.Last_input.write().unwrap();

                        Last_input.0.X = x as i16;
                        Last_input.0.Y = y as i16;
                        Last_input.1 = Generics::Touch_type::Pressed;
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
                        let mut Last_input = self.Last_input.write().unwrap();

                        Last_input.1 = Generics::Touch_type::Released;
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
                        let mut Last_input = self.Last_input.write().unwrap();

                        Last_input.0.X = x as i16;
                        Last_input.0.Y = y as i16;
                    }
                }
                _ => {}
            };
        }
    }
}

impl Generics::Input_traits for Pointer_type {
    fn Get_latest_input(&self) -> (Generics::Point_type, Generics::Touch_type) {
        *self.Last_input.read().unwrap()
    }
}

pub fn New_touchscreen(Size: Generics::Point_type) -> Result_type<(Screen_type, Pointer_type)> {
    let Context = sdl2::init()?;

    let Video_subsystem = Context.video()?;

    let Window = Video_subsystem
        .window("Xila", Size.X as u32, Size.Y as u32)
        .position_centered()
        .build()?;

    let Event_pump = Context.event_pump()?;

    let Pointer = Pointer_type::New(Window.id(), Event_pump);

    let Screen = Screen_type::New(Window)?;

    Ok((Screen, Pointer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_touchscreen() {
        const Horizontal_resolution: u32 = 800;
        const Vertical_resolution: u32 = 480;

        let Touchscreen = New_touchscreen(Generics::Point_type::New(
            Horizontal_resolution as i16,
            Vertical_resolution as i16,
        ));

        assert!(Touchscreen.is_ok());

        let (_, _) = Touchscreen.unwrap();

        unsafe {
            sdl2::sys::SDL_Quit(); // Force SDL2 to quit to avoid conflicts with other tests.
        }
    }
}
