use std::time::Duration;

use winit::{
    event_loop::EventLoop,
    platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland},
};
use Graphics::{Input_data_type, Key_type, Point_type, Screen_write_data_type, State_type};

use super::Window_type;

pub struct Inner_type(Window_type, EventLoop<()>);

unsafe impl Sync for Inner_type {}

unsafe impl Send for Inner_type {}

impl Inner_type {
    pub fn new(resolution: Point_type) -> Result<Self, String> {
        let mut event_loop = EventLoop::builder()
            //  .with_wayland()
            .with_any_thread(true)
            .build()
            .map_err(|error| format!("Error building event loop: {error:?}"))?;

        let mut window = Window_type::new(resolution);

        event_loop.pump_app_events(None, &mut window);

        Ok(Self(window, event_loop))
    }

    pub fn get_resolution(&self) -> Option<Point_type> {
        self.0.get_resolution()
    }

    pub fn draw(&mut self, data: &Screen_write_data_type) -> Result<(), String> {
        self.0.draw(data)
    }

    pub fn get_pointer_data(&mut self) -> Result<&Input_data_type, String> {
        self.1.pump_app_events(Some(Duration::ZERO), &mut self.0);

        Ok(self.0.get_pointer_data())
    }

    pub fn pop_keyboard_data(&mut self) -> Option<(State_type, Key_type, bool)> {
        self.1.pump_app_events(Some(Duration::ZERO), &mut self.0);

        self.0.pop_keyboard_data()
    }
}
