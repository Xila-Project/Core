mod event_loop;
mod inner_window;
mod keyboard;
mod pointer;
mod screen;
mod window;

use graphics::{InputData, Key, Point, State};
use synchronization::rwlock::RwLock;
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use inner_window::*;
use keyboard::*;
use pointer::*;
use screen::*;
use window::*;

use crate::window_screen::event_loop::Runner;

static KEYBOARD_CHANNEL: Channel<CriticalSectionRawMutex, (Key, State), 64> = Channel::new();
static POINTER_RWLOCK: RwLock<CriticalSectionRawMutex, InputData> =
    RwLock::new(InputData::default_constant());
static INNER_WINDOW: InnerWindow = InnerWindow::new();

pub async fn new(
    resolution: Point,
) -> Result<
    (
        ScreenDevice<'static>,
        PointerDevice,
        KeyboardDevice<CriticalSectionRawMutex, 64>,
        Runner<'static>,
    ),
    String,
> {
    let window = Window::new(
        resolution,
        &INNER_WINDOW,
        KEYBOARD_CHANNEL.sender(),
        &POINTER_RWLOCK,
    );

    let event_loop = Runner::new(window);

    let screen_device = ScreenDevice::new(&INNER_WINDOW);

    let pointer_device = PointerDevice::new(&POINTER_RWLOCK);

    let keyboard_device = KeyboardDevice::new(KEYBOARD_CHANNEL.receiver());

    Ok((screen_device, pointer_device, keyboard_device, event_loop))
}
