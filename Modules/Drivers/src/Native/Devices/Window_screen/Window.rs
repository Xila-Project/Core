use std::collections::VecDeque;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use Graphics::{
    Color_RGBA8888_type, Input_data_type, Key_type, Point_type, Screen_write_data_type, State_type,
};

#[derive(Default)]
pub struct Window_type {
    Resolution: Point_type,
    Window: Option<Window>,
    Pixels: Option<Pixels>,
    Pointer_data: Input_data_type,
    Keyboard_data: VecDeque<(State_type, Key_type)>,
}

impl Window_type {
    pub fn New(Resolution: Point_type) -> Self {
        Self {
            Resolution,
            Keyboard_data: VecDeque::with_capacity(16),
            ..Default::default()
        }
    }

    pub fn Get_pointer_data(&self) -> &Input_data_type {
        &self.Pointer_data
    }

    pub fn Pop_keyboard_data(&mut self) -> Option<(State_type, Key_type, bool)> {
        let (State, Key) = self.Keyboard_data.pop_front()?;

        let Continue = self.Keyboard_data.is_empty();

        Some((State, Key, Continue))
    }

    pub fn Get_resolution(&self) -> Option<Point_type> {
        self.Window.as_ref().map(|Window| {
            let Size = Window.inner_size();
            Point_type::New(Size.width as i16, Size.height as i16)
        })
    }

    pub fn Draw(&mut self, Data: &Screen_write_data_type) -> Result<(), String> {
        let Frame_width = self.Resolution.Get_x() as usize;
        let Data_area = Data.Get_area();

        let Point_1 = Data_area.Get_point_1();
        let Point_2 = Data_area.Get_point_2();

        let Pixels = self
            .Pixels
            .as_mut()
            .ok_or_else(|| "Pixels is None.".to_string())?;

        let Frame = Pixels.frame_mut();
        let Frame = unsafe {
            core::slice::from_raw_parts_mut(
                Frame.as_mut_ptr() as *mut Color_RGBA8888_type,
                Frame.len() / size_of::<Color_RGBA8888_type>(),
            )
        };

        let Data_buffer = Data.Get_buffer();

        let Frame_x_start = Point_1.Get_x() as usize;
        let Frame_y_start = Point_1.Get_y() as usize;
        let Width = (Point_2.Get_x() - Point_1.Get_x() + 1) as usize;
        let Height = (Point_2.Get_y() - Point_1.Get_y() + 1) as usize;

        for (y, Data_row) in Data_buffer.chunks(Width).enumerate().take(Height) {
            let Frame_row_start = (Frame_y_start + y) * Frame_width + Frame_x_start;
            let Frame_row_end = Frame_row_start + Width;
            let Frame_row = &mut Frame[Frame_row_start..Frame_row_end];

            Frame_row
                .iter_mut()
                .zip(Data_row.iter())
                .for_each(|(Destination, &Source)| {
                    let Source = Color_RGBA8888_type::From_RGB565(Source);
                    *Destination = Source;
                });
        }

        // - Request a redraw.
        self.Window
            .as_ref()
            .ok_or_else(|| "Window is None.".to_string())?
            .request_redraw();

        Ok(())
    }
}

impl ApplicationHandler for Window_type {
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(Window) = &self.Window {
            Window.request_redraw();
        }
    }

    fn resumed(&mut self, Event_loop: &ActiveEventLoop) {
        let Window = {
            let Size = LogicalSize::new(
                self.Resolution.Get_x() as f64,
                self.Resolution.Get_y() as f64,
            );

            let Window_attributes = Window::default_attributes()
                .with_title("Xila")
                .with_inner_size(Size)
                .with_min_inner_size(Size);

            Event_loop.create_window(Window_attributes).unwrap()
        };

        let Pixels = {
            let Surface_texture = SurfaceTexture::new(
                self.Resolution.Get_x() as u32,
                self.Resolution.Get_y() as u32,
                &Window,
            );

            Pixels::new(
                self.Resolution.Get_x() as u32,
                self.Resolution.Get_y() as u32,
                Surface_texture,
            )
            .unwrap()
        };

        self.Window = Some(Window);
        self.Pixels = Some(Pixels);
    }

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        Window_identifier: WindowId,
        Event: WindowEvent,
    ) {
        let Window = if let Some(Window) = &self.Window {
            Window
        } else {
            return;
        };

        if Window_identifier != Window.id() {
            return;
        }

        match Event {
            WindowEvent::RedrawRequested => {
                if let Some(Pixels) = &mut self.Pixels {
                    Pixels.render().unwrap();
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event: Event,
                is_synthetic: _,
            } => {
                if let Some(Text) = Event.text {
                    if Event.state == ElementState::Pressed {
                        let Key = Text.as_bytes()[0];

                        self.Keyboard_data
                            .push_back((State_type::Pressed, Key_type::Character(Key)));
                        self.Keyboard_data
                            .push_back((State_type::Released, Key_type::Character(Key)));
                    }
                } else if let winit::keyboard::Key::Named(Key) = Event.logical_key {
                    let State = match Event.state {
                        ElementState::Pressed => State_type::Pressed,
                        ElementState::Released => State_type::Released,
                    };

                    let Key = match Key {
                        winit::keyboard::NamedKey::ArrowUp => Key_type::Up,
                        winit::keyboard::NamedKey::ArrowDown => Key_type::Down,
                        winit::keyboard::NamedKey::ArrowLeft => Key_type::Left,
                        winit::keyboard::NamedKey::ArrowRight => Key_type::Right,
                        winit::keyboard::NamedKey::Escape => Key_type::Escape,
                        winit::keyboard::NamedKey::Delete => Key_type::Delete,
                        winit::keyboard::NamedKey::Backspace => Key_type::Backspace,
                        winit::keyboard::NamedKey::Enter => Key_type::Enter,
                        winit::keyboard::NamedKey::NavigateNext => Key_type::Next,
                        winit::keyboard::NamedKey::NavigatePrevious => Key_type::Previous,
                        winit::keyboard::NamedKey::Home => Key_type::Home,
                        winit::keyboard::NamedKey::End => Key_type::End,
                        _ => Key_type::Character(0),
                    };

                    self.Keyboard_data.push_back((State, Key));
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position: Position,
            } => self
                .Pointer_data
                .Set_point((Position.x as i16, Position.y as i16).into()),
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } => match state {
                ElementState::Pressed => {
                    self.Pointer_data.Set_state(State_type::Pressed);
                }

                ElementState::Released => {
                    self.Pointer_data.Set_state(State_type::Released);
                }
            },
            _ => {}
        }
    }
}
