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
    pub fn New(Resolution: Point_type) -> Result<Self, String> {
        let mut Event_loop = EventLoop::builder()
            //  .with_wayland()
            .with_any_thread(true)
            .build()
            .map_err(|Error| format!("Error building event loop: {:?}", Error))?;

        let mut Window = Window_type::New(Resolution);

        Event_loop.pump_app_events(None, &mut Window);

        Ok(Self(Window, Event_loop))
    }

    pub fn Get_resolution(&self) -> Option<Point_type> {
        self.0.Get_resolution()
    }

    pub fn Draw(&mut self, Data: &Screen_write_data_type) -> Result<(), String> {
        self.0.Draw(Data)
    }

    pub fn Get_pointer_data(&mut self) -> Result<&Input_data_type, String> {
        self.1.pump_app_events(Some(Duration::ZERO), &mut self.0);

        Ok(self.0.Get_pointer_data())
    }

    pub fn Pop_keyboard_data(&mut self) -> Option<(State_type, Key_type, bool)> {
        self.1.pump_app_events(Some(Duration::ZERO), &mut self.0);

        self.0.Pop_keyboard_data()
    }
}
