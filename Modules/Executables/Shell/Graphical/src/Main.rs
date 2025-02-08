use core::num::NonZeroUsize;
use std::time::Duration;

use Executable::Standard_type;
use File_system::Path_type;

use crate::{
    Desk::Desk_type, Error::Error_type, Home::Home_type, Layout::Layout_type, Login::Login_type,
    Shell_type, Shortcut::Shortcut_type,
};

pub fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::New(Standard).Main(Arguments)
}

impl Shell_type {
    pub fn New(Standard: Standard_type) -> Self {
        let Layout = Layout_type::New().unwrap();

        let Login = Login_type::New().unwrap();

        Self {
            _Standard: Standard,
            Layout,
            Desk: None,
            Running: true,
            _Home: None,
            Login: Some(Login),
        }
    }

    pub fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        let Arguments: Vec<&str> = Arguments.split_whitespace().collect();

        if Arguments.first() == Some(&"add_shortcut") {
            if Arguments.len() != 2 {
                return Err(Error_type::Missing_arguments.into());
            }

            Shortcut_type::Add(Path_type::From_str(Arguments[1]))?;
        }

        while self.Running {
            self.Layout.Loop();

            if let Some(Login) = &mut self.Login {
                Login.Event_handler();

                if let Some(User) = Login.Get_logged_user() {
                    let User_name = Users::Get_instance().Get_user_name(User).unwrap();

                    Task::Get_instance()
                        .Set_environment_variable(
                            self._Standard.Get_task(),
                            "User",
                            User_name.as_str(),
                        )
                        .map_err(Error_type::Failed_to_set_environment_variable)?;

                    self.Desk = Some(Desk_type::New()?);

                    if let Some(Desk) = &mut self.Desk {
                        self._Home = Some(Home_type::New(Desk.Get_window_object())?);
                    }

                    self.Login = None;
                }
            }

            if let Some(Desk) = &mut self.Desk {
                if !Desk.Is_hidden() {
                    Desk.Event_handler();
                }
            }

            Task::Manager_type::Sleep(Duration::from_millis(20));
        }

        Ok(())
    }
}
