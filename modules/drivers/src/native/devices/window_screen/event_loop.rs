use std::time::Duration;

use winit::platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland};

use crate::native::window_screen::window::Window;

pub struct Runner<'a>(Window<'a>, winit::event_loop::EventLoop<()>);

impl<'a> Runner<'a> {
    pub fn new(window: Window<'a>) -> Self {
        let event_loop = winit::event_loop::EventLoop::builder()
            .with_any_thread(true)
            .build()
            .unwrap();

        let mut event_loop = Self(window, event_loop);

        event_loop.run_once();

        event_loop
    }

    pub fn run_once(&mut self) {
        self.1.pump_app_events(None, &mut self.0);
    }

    pub async fn run(&mut self) {
        loop {
            self.run_once();
            task::Manager::sleep(Duration::from_millis(16)).await;
        }
    }
}
