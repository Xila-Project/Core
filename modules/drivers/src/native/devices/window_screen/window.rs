use std::collections::VecDeque;

use graphics::{ColorRGBA8888, InputData, Key, Point, ScreenWriteData, State};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{self, WindowId},
};

#[derive(Default)]
pub struct Window {
    resolution: Point,
    window: Option<window::Window>,
    pixels: Option<Pixels>,
    pointer_data: InputData,
    keyboard_data: VecDeque<(State, Key)>,
}

impl Window {
    pub fn new(resolution: Point) -> Self {
        Self {
            resolution,
            keyboard_data: VecDeque::with_capacity(16),
            ..Default::default()
        }
    }

    pub fn get_pointer_data(&self) -> &InputData {
        &self.pointer_data
    }

    pub fn pop_keyboard_data(&mut self) -> Option<(State, Key, bool)> {
        let (state, key) = self.keyboard_data.pop_front()?;

        let r#continue = self.keyboard_data.is_empty();

        Some((state, key, r#continue))
    }

    pub fn get_resolution(&self) -> Option<Point> {
        self.window.as_ref().map(|window| {
            let size = window.inner_size();
            Point::new(size.width as i16, size.height as i16)
        })
    }

    pub fn draw(&mut self, data: &ScreenWriteData) -> Result<(), String> {
        let frame_width = self.resolution.get_x() as usize;
        let data_area = data.get_area();

        let point_1 = data_area.get_point_1();
        let point_2 = data_area.get_point_2();

        let pixels = self
            .pixels
            .as_mut()
            .ok_or_else(|| "Pixels is None.".to_string())?;

        let frame = pixels.frame_mut();
        let frame = unsafe {
            core::slice::from_raw_parts_mut(
                frame.as_mut_ptr() as *mut ColorRGBA8888,
                frame.len() / size_of::<ColorRGBA8888>(),
            )
        };

        let data_buffer = data.get_buffer();

        let frame_x_start = point_1.get_x() as usize;
        let frame_y_start = point_1.get_y() as usize;
        let width = (point_2.get_x() - point_1.get_x() + 1) as usize;
        let height = (point_2.get_y() - point_1.get_y() + 1) as usize;

        for (y, data_row) in data_buffer.chunks(width).enumerate().take(height) {
            let frame_row_start = (frame_y_start + y) * frame_width + frame_x_start;
            let frame_row_end = frame_row_start + width;
            let frame_row = &mut frame[frame_row_start..frame_row_end];

            frame_row
                .iter_mut()
                .zip(data_row.iter())
                .for_each(|(destination, &source)| {
                    *destination = source.into();
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

impl ApplicationHandler for Window {
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let size = LogicalSize::new(
                self.resolution.get_x() as f64,
                self.resolution.get_y() as f64,
            );

            let window_attributes = window::Window::default_attributes()
                .with_title("Xila")
                .with_inner_size(size)
                .with_min_inner_size(size);

            event_loop.create_window(window_attributes).unwrap()
        };

        let pixels = {
            let surface_texture = SurfaceTexture::new(
                self.resolution.get_x() as u32,
                self.resolution.get_y() as u32,
                &window,
            );

            Pixels::new(
                self.resolution.get_x() as u32,
                self.resolution.get_y() as u32,
                surface_texture,
            )
            .unwrap()
        };

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        window_identifier: WindowId,
        event: WindowEvent,
    ) {
        let window = if let Some(window) = &self.window {
            window
        } else {
            return;
        };

        if window_identifier != window.id() {
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
                if let Some(text) = event.text {
                    if event.state == ElementState::Pressed {
                        let key = text.as_bytes()[0];

                        self.keyboard_data
                            .push_back((State::Pressed, Key::Character(key)));
                        self.keyboard_data
                            .push_back((State::Released, Key::Character(key)));
                    }
                } else if let winit::keyboard::Key::Named(key) = event.logical_key {
                    let state = match event.state {
                        ElementState::Pressed => State::Pressed,
                        ElementState::Released => State::Released,
                    };

                    let key = match key {
                        winit::keyboard::NamedKey::ArrowUp => Key::Up,
                        winit::keyboard::NamedKey::ArrowDown => Key::Down,
                        winit::keyboard::NamedKey::ArrowLeft => Key::Left,
                        winit::keyboard::NamedKey::ArrowRight => Key::Right,
                        winit::keyboard::NamedKey::Escape => Key::Escape,
                        winit::keyboard::NamedKey::Delete => Key::Delete,
                        winit::keyboard::NamedKey::Backspace => Key::Backspace,
                        winit::keyboard::NamedKey::Enter => Key::Enter,
                        winit::keyboard::NamedKey::NavigateNext => Key::Next,
                        winit::keyboard::NamedKey::NavigatePrevious => Key::Previous,
                        winit::keyboard::NamedKey::Home => Key::Home,
                        winit::keyboard::NamedKey::End => Key::End,
                        _ => Key::Character(0),
                    };

                    self.keyboard_data.push_back((state, key));
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => self
                .pointer_data
                .set_point((position.x as i16, position.y as i16).into()),
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } => match state {
                ElementState::Pressed => {
                    self.pointer_data.set_state(State::Pressed);
                }

                ElementState::Released => {
                    self.pointer_data.set_state(State::Released);
                }
            },
            _ => {}
        }
    }
}
