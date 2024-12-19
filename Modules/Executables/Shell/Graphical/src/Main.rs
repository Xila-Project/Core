use core::num::NonZeroUsize;
use std::time::Duration;

use Executable::Standard_type;

use crate::{Desk::Desk_type, Home::Home_type, Layout::Layout_type, Shell_type};

pub fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::New(Standard).Main(Arguments)
}

impl Shell_type {
    pub fn New(Standard: Standard_type) -> Self {
        let Layout = Layout_type::New().unwrap();

        let Desk = Desk_type::New().unwrap();

        let Home = Home_type::New(Desk.Get_window_object()).unwrap();

        Self {
            _Standard: Standard,
            Layout,
            Desk,
            Running: true,
            _Home: Home,
        }
    }

    pub fn Main(&mut self, _: String) -> Result<(), NonZeroUsize> {
        while self.Running {
            self.Layout.Loop();

            if !self.Desk.Is_hidden() {
                self.Desk.Event_handler();
            }

            Task::Manager_type::Sleep(Duration::from_millis(20));
        }

        Ok(())
    }
}
