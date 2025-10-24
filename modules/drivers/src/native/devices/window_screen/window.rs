use alloc::sync::Arc;

use embassy_sync::rwlock::RwLock;
use graphics::{InputData, Key, Point, State};
use pixels::{Pixels, SurfaceTexture};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{self, WindowId},
};

use super::InnerWindow;

pub struct Window<'a> {
    resolution: Point,
    inner_window: &'a InnerWindow,
    keyboard_receiver: Sender<'static, CriticalSectionRawMutex, (Key, State), 64>,
    pointer_rwlock: &'static RwLock<CriticalSectionRawMutex, InputData>,
}

impl<'a> Window<'a> {
    pub const fn new(
        resolution: Point,
        inner_window: &'a InnerWindow,
        keyboard_receiver: Sender<'static, CriticalSectionRawMutex, (Key, State), 64>,
        pointer_rwlock: &'static RwLock<CriticalSectionRawMutex, InputData>,
    ) -> Self {
        Self {
            resolution,
            inner_window,
            keyboard_receiver,
            pointer_rwlock,
        }
    }
}

impl<'a> ApplicationHandler for Window<'a> {
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

            let window = event_loop.create_window(window_attributes).unwrap();

            Arc::new(window)
        };

        let pixels = {
            let surface_texture = SurfaceTexture::new(
                self.resolution.get_x() as u32,
                self.resolution.get_y() as u32,
                window.clone(),
            );

            Pixels::new(
                self.resolution.get_x() as u32,
                self.resolution.get_y() as u32,
                surface_texture,
            )
            .unwrap()
        };

        futures::block_on(self.inner_window.replace(window, pixels));
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                futures::block_on(self.inner_window.render()).unwrap();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let Some(text) = event.text {
                    if event.state == ElementState::Pressed {
                        let key = text.as_bytes()[0];

                        let key = match key {
                            b'\n' | b'\r' => Key::Enter,
                            _ => Key::Character(key),
                        };

                        if let Err(e) = self.keyboard_receiver.try_send((key, State::Pressed)) {
                            log::error!("Failed to send key event: {:?}", e);
                            return;
                        }
                        if let Err(e) = self.keyboard_receiver.try_send((key, State::Released)) {
                            log::error!("Failed to send key event: {:?}", e);
                        }
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

                    if let Err(e) = self.keyboard_receiver.try_send((key, state)) {
                        log::error!("Failed to send key event: {:?}", e);
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                if let Ok(mut pointer_data) = self.pointer_rwlock.try_write() {
                    pointer_data.set_point((position.x as i16, position.y as i16).into());
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } => {
                if let Ok(mut pointer_data) = self.pointer_rwlock.try_write() {
                    match state {
                        ElementState::Pressed => {
                            pointer_data.set_state(State::Pressed);
                        }

                        ElementState::Released => {
                            pointer_data.set_state(State::Released);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
