use std::time::Duration;

use graphics::{Input_data_type, Key, Point, ScreenWriteData, State};
use winit::{
    event_loop::EventLoop,
    platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland},
};

use super::Window;

pub struct Inner(Window, EventLoop<()>);

unsafe impl Sync for Inner {}

unsafe impl Send for Inner {}

impl Inner {
    pub fn new(resolution: Point) -> Result<Self, String> {
        let mut event_loop = EventLoop::builder()
            //  .with_wayland()
            .with_any_thread(true)
            .build()
            .map_err(|error| format!("Error building event loop: {error:?}"))?;

        let mut window = Window::new(resolution);

        event_loop.pump_app_events(None, &mut window);

        Ok(Self(window, event_loop))
    }

    pub fn get_resolution(&self) -> Option<Point> {
        self.0.get_resolution()
    }

    pub fn draw(&mut self, data: &ScreenWriteData) -> Result<(), String> {
        self.0.draw(data)
    }

    pub fn get_pointer_data(&mut self) -> Result<&Input_data_type, String> {
        self.1.pump_app_events(Some(Duration::ZERO), &mut self.0);

        Ok(self.0.get_pointer_data())
    }

    pub fn pop_keyboard_data(&mut self) -> Option<(State, Key, bool)> {
        self.1.pump_app_events(Some(Duration::ZERO), &mut self.0);

        self.0.pop_keyboard_data()
    }
}
