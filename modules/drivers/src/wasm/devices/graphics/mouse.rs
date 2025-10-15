use alloc::sync::Arc;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use file_system::{DeviceTrait, Size};
use futures::block_on;
use graphics::{InputData, Point, State};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlCanvasElement, MouseEvent};

struct Inner {
    position: Point,
    state: State,
}

pub struct MouseDevice(Arc<RwLock<CriticalSectionRawMutex, Inner>>);

impl MouseDevice {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let inner = Arc::new(RwLock::new(Inner {
            position: Point::new(0, 0),
            state: State::default(),
        }));

        let inner_clone = inner.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut inner = block_on(inner_clone.write());

            inner.position = Point::new(event.client_x() as _, event.client_y() as _);
            inner.state = if event.buttons() == 0 {
                State::default()
            } else {
                State::Pressed
            };
        }) as Box<dyn FnMut(MouseEvent)>);

        canvas
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add mousemove event listener")?;
        canvas
            .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add mouseup event listener")?;
        canvas
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add mousemove event listener")?;
        canvas
            .add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())
            .map_err(|_| "Failed to add mouseleave event listener")?;
        closure.forget(); // Prevent memory leak by keeping the closure alive

        Ok(Self(inner))
    }
}

impl DeviceTrait for MouseDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        let data: &mut InputData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        let inner = block_on(self.0.read());

        data.set(inner.position, inner.state);

        Ok(Size::new(0))
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(size_of::<InputData>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
