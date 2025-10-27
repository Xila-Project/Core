use alloc::{boxed::Box, string::String};
use file_system::DeviceTrait;

use graphics::{InputData, Key, State};
use synchronization::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, channel::Sender,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::KeyboardEvent;

#[derive(Debug)]
pub struct KeyboardDevice(Box<Channel<CriticalSectionRawMutex, (Key, State), 64>>);

impl KeyboardDevice {
    fn handle_key_press<const N: usize>(
        sender: Sender<'_, CriticalSectionRawMutex, (Key, State), N>,
        key: &str,
        pressed: bool,
    ) {
        let key = match map_key(key) {
            Some(k) => k,
            None => {
                log::warning!("Unsupported key: {}", key);
                return;
            }
        };

        let state = if pressed {
            State::Pressed
        } else {
            State::Released
        };

        if let Err(e) = sender.try_send((key, state)) {
            log::error!("Failed to send key event: {:?}", e);
        }
    }

    pub fn new() -> Result<Self, String> {
        let document = web_sys::window()
            .ok_or("Failed to get window")?
            .document()
            .ok_or("Failed to get document")?;

        let inner = Self(Box::new(Channel::new()));

        let sender = inner.0.sender();

        let key_down_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            Self::handle_key_press(sender, &event.key(), true);
            Self::handle_key_press(sender, &event.key(), false);
        }) as Box<dyn FnMut(KeyboardEvent)>);

        document
            .add_event_listener_with_callback("keydown", key_down_closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add keydown event listener")?;
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

        if let Ok((key, state)) = self.0.receiver().try_receive() {
            data.set_key(key);
            data.set_state(state);
        }

        data.set_continue(!self.0.is_empty());

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
