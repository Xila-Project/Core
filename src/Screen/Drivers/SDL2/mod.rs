use crate::Screen::Generics;

use Generics::Coordinates_type;
use sdl2::{event, keyboard, pixels, render, video};
use std::sync::RwLock;

pub struct Screen_type(RwLock<render::Canvas<video::Window>>);

impl Screen_type {
    pub fn New(Size : Coordinates_type) -> Result<Self, ()> {
        let Context = match sdl2::init() {
            Ok(Context) => Context,
            Err(_) => return Err(()),
        };

        let Video_subsystem = match Context.video() {
            Ok(Video_subsystem) => Video_subsystem,
            Err(_) => return Err(()),
        };

        let Window = match Video_subsystem
            .window("Xila", Size.X as u32, Size.Y as u32)
            .position_centered()
            .build()
        {
            Ok(Window) => Window,
            Err(_) => return Err(()),
        };

        let mut Canvas = match Window.into_canvas().build() {
            Ok(Canvas) => Canvas,
            Err(_) => return Err(()),
        };

        Canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        Canvas.clear();
        Canvas.present();

        Ok(Self(RwLock::new(Canvas)))
    }
}

impl<const B: usize> Generics::Screen_traits<B> for Screen_type {
    fn Update(&mut self, Refresh_area: &Generics::Refresh_area_type<B>) {
        let mut Canvas = match self.0.write() {
            Ok(Canvas) => Canvas,
            Err(_) => return,
        };

        for Y in
            Refresh_area.Area.Position.Y..Refresh_area.Area.Position.Y + Refresh_area.Area.Size.Y
        {
            for X in Refresh_area.Area.Position.X
                ..Refresh_area.Area.Position.X + Refresh_area.Area.Size.X
            {
                let Color = &Refresh_area.Buffer[(Y * Refresh_area.Area.Size.X + X) as usize];

                print!("At : {}, {} - Print color : {}, {}, {}\n", X, Y, Color.Red, Color.Green, Color.Blue);
                Canvas.set_draw_color(pixels::Color::RGB(Color.Red, Color.Green, Color.Blue));
                let _ = Canvas.draw_point(sdl2::rect::Point::new(X as i32, Y as i32));
            }
        }

        Canvas.present();
    }

    fn Get_resolution(&self) -> Result<Generics::Coordinates_type, ()> {
        match self.0.read() {
            Ok(Canvas) => match Canvas.output_size() {
                Ok(Resolution) => Ok(Generics::Coordinates_type {
                    X: Resolution.0 as i16,
                    Y: Resolution.1 as i16,
                }),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}
