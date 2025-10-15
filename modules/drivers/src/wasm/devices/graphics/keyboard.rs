use alloc::{collections::VecDeque, sync::Arc};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use file_system::DeviceTrait;
use futures::block_on;
use graphics::{InputData, Key, State};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::KeyboardEvent;

#[derive(Clone, Debug)]
pub struct KeyboardDevice(Arc<RwLock<CriticalSectionRawMutex, VecDeque<(Key, State)>>>);

impl KeyboardDevice {
    fn handle_key_press(&self, key: &str, pressed: bool) {
        let key = match map_key(key) {
            Some(k) => k,
            None => {
                log::Warning!("Unsupported key: {}", key);
                return;
            }
        };

        let state = if pressed {
            State::Pressed
        } else {
            State::Released
        };

        let mut inner = block_on(self.0.write());
        inner.push_back((key, state));
    }

    pub fn new() -> Result<Self, String> {
        let document = web_sys::window()
            .ok_or("Failed to get window")?
            .document()
            .ok_or("Failed to get document")?;

        let inner = Self(Arc::new(RwLock::new(VecDeque::new())));

        let inner_clone = inner.clone();
        let key_down_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            inner_clone.handle_key_press(&event.key(), true);
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let inner_clone = inner.clone();
        let key_up_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            inner_clone.handle_key_press(&event.key(), false);
        }) as Box<dyn FnMut(KeyboardEvent)>);

        document
            .add_event_listener_with_callback("keydown", key_down_closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add keydown event listener")?;
        document
            .add_event_listener_with_callback("keyup", key_up_closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add keyup event listener")?;
        key_up_closure.forget(); // Prevent memory leak by keeping the closure alive
        key_down_closure.forget(); // Prevent memory leak by keeping the closure alive

        Ok(inner)
    }
}

impl DeviceTrait for KeyboardDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        //log::information!("Keyboard read: {:?}", buffer);

        let data: &mut InputData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        let mut inner = block_on(self.0.write());

        if let Some(key) = inner.pop_front() {
            data.set_key(key.0);
            data.set_state(key.1);
        }

        data.set_continue(inner.is_empty());

        //log::information!("Keyboard read: {:?}", data);

        Ok(file_system::Size::new(buffer.len() as u64))
    }

    fn write(&self, _: &[u8]) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        Ok(size_of::<InputData>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}

fn map_key(key: &str) -> Option<Key> {
    if key.len() == 1 {
        let character = key.chars().next()?;

        if character.is_ascii() {
            return Some(Key::Character(character as u8));
        }
    }

    let key = match key {
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowRight" => Key::Right,
        "ArrowLeft" => Key::Left,
        "Escape" => Key::Escape,
        "Delete" => Key::Delete,
        "Backspace" => Key::Backspace,
        "Enter" => Key::Enter,
        "PageDown" => Key::Next,
        "PageUp" => Key::Previous,
        "Home" => Key::Home,
        "End" => Key::End,
        _ => return None,
    };

    Some(key)
}
