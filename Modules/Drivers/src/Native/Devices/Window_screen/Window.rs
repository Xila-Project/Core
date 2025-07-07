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
    resolution: Point_type,
    window: Option<Window>,
    pixels: Option<Pixels>,
    pointer_data: Input_data_type,
    keyboard_data: VecDeque<(State_type, Key_type)>,
}

impl Window_type {
    pub fn new(resolution: Point_type) -> Self {
        Self {
            resolution,
            keyboard_data: VecDeque::with_capacity(16),
            ..Default::default()
        }
    }

    pub fn Get_pointer_data(&self) -> &Input_data_type {
        &self.pointer_data
    }

    pub fn Pop_keyboard_data(&mut self) -> Option<(State_type, Key_type, bool)> {
        let (state, key) = self.keyboard_data.pop_front()?;

        let Continue = self.keyboard_data.is_empty();

        Some((state, key, Continue))
    }

    pub fn Get_resolution(&self) -> Option<Point_type> {
        self.window.as_ref().map(|window| {
            let size = window.inner_size();
            Point_type::new(size.width as i16, size.height as i16)
        })
    }

    pub fn Draw(&mut self, Data: &Screen_write_data_type) -> Result<(), String> {
        let frame_width = self.resolution.Get_x() as usize;
        let data_area = Data.get_area();

        let Point_1 = data_area.Get_point_1();
        let point_2 = data_area.Get_point_2();

        let Pixels = self
            .pixels
            .as_mut()
            .ok_or_else(|| "Pixels is None.".to_string())?;

        let Frame = Pixels.frame_mut();
        let frame = unsafe {
            core::slice::from_raw_parts_mut(
                Frame.as_mut_ptr() as *mut Color_RGBA8888_type,
                Frame.len() / size_of::<Color_RGBA8888_type>(),
            )
        };

        let Data_buffer = Data.Get_buffer();

        let Frame_x_start = Point_1.Get_x() as usize;
        let frame_y_start = Point_1.Get_y() as usize;
        let width = (point_2.Get_x() - Point_1.Get_x() + 1) as usize;
        let height = (point_2.Get_y() - Point_1.Get_y() + 1) as usize;

        for (y, Data_row) in Data_buffer.chunks(width).enumerate().take(height) {
            let frame_row_start = (frame_y_start + y) * frame_width + Frame_x_start;
            let frame_row_end = frame_row_start + width;
            let frame_row = &mut frame[frame_row_start..frame_row_end];

            frame_row
                .iter_mut()
                .zip(Data_row.iter())
                .for_each(|(destination, &Source)| {
                    let source = Color_RGBA8888_type::From_RGB565(Source);
                    *destination = source;
                });
        }

        // - Request a redraw.
        self.window
            .as_ref()
            .ok_or_else(|| "Window is None.".to_string())?
            .request_redraw();

        Ok(())
    }
}

impl ApplicationHandler for Window_type {
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resumed(&mut self, Event_loop: &ActiveEventLoop) {
        let window = {
            let size = LogicalSize::new(
                self.resolution.Get_x() as f64,
                self.resolution.Get_y() as f64,
            );

            let Window_attributes = Window::default_attributes()
                .with_title("Xila")
                .with_inner_size(size)
                .with_min_inner_size(size);

            Event_loop.create_window(Window_attributes).unwrap()
        };

        let Pixels = {
            let surface_texture = SurfaceTexture::new(
                self.resolution.Get_x() as u32,
                self.resolution.Get_y() as u32,
                &window,
            );

            Pixels::new(
                self.resolution.Get_x() as u32,
                self.resolution.Get_y() as u32,
                surface_texture,
            )
            .unwrap()
        };

        self.window = Some(window);
        self.pixels = Some(Pixels);
    }

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        window_identifier: WindowId,
        event: WindowEvent,
    ) {
        let Window = if let Some(Window) = &self.window {
            Window
        } else {
            return;
        };

        if window_identifier != Window.id() {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    pixels.render().unwrap();
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let Some(Text) = event.text {
                    if event.state == ElementState::Pressed {
                        let key = Text.as_bytes()[0];

                        self.keyboard_data
                            .push_back((State_type::Pressed, Key_type::Character(key)));
                        self.keyboard_data
                            .push_back((State_type::Released, Key_type::Character(key)));
                    }
                } else if let winit::keyboard::Key::Named(Key) = event.logical_key {
                    let state = match event.state {
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

                    self.keyboard_data.push_back((state, Key));
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => self
                .pointer_data
                .Set_point((position.x as i16, position.y as i16).into()),
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } => match state {
                ElementState::Pressed => {
                    self.pointer_data.Set_state(State_type::Pressed);
                }

                ElementState::Released => {
                    self.pointer_data.Set_state(State_type::Released);
                }
            },
            _ => {}
        }
    }
}
