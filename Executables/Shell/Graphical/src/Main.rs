use core::num::NonZeroUsize;
use core::time::Duration;

use alloc::{boxed::Box, string::String, vec::Vec};
use Executable::Standard_type;
use File_system::Path_type;

use crate::{
    Desk::Desk_type, Error::Error_type, Home::Home_type, Layout::Layout_type, Login::Login_type,
    Shell_type, Shortcut::Shortcut_type,
};

pub async fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::new(Standard).await.Main(Arguments).await
}

impl Shell_type {
    pub async fn new(Standard: Standard_type) -> Self {
        let layout = Layout_type::New().await.unwrap();

        let Login = Box::new(Login_type::new().await.unwrap());

        Self {
            _standard: Standard,
            layout,
            desk: None,
            running: true,
            _home: None,
            login: Some(Login),
        }
    }

    pub async fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        let arguments: Vec<&str> = Arguments.split_whitespace().collect();

        if arguments.first() == Some(&"add_shortcut") {
            if arguments.len() != 2 {
                return Err(Error_type::Missing_arguments.into());
            }

            Shortcut_type::add(Path_type::From_str(arguments[1])).await?;
        }

        while self.running {
            self.layout.Loop().await;

            if let Some(Login) = &mut self.login {
                Login.Event_handler().await;

                if let Some(User) = Login.Get_logged_user() {
                    let user_name = Users::Get_instance().Get_user_name(User).await.unwrap();

                    Task::Get_instance()
                        .Set_environment_variable(
                            self._standard.Get_task(),
                            "User",
                            user_name.as_str(),
                        )
                        .await
                        .map_err(Error_type::Failed_to_set_environment_variable)?;

                    self.desk = Some(Box::new(
                        Desk_type::New(self.layout.Get_windows_parent()).await?,
                    ));

                    if let Some(Desk) = &mut self.desk {
                        self._home =
                            Some(Box::new(Home_type::new(Desk.Get_window_object()).await?));
                    }

                    self.login = None;
                }
            }

            if let Some(Desk) = &mut self.desk {
                if !Desk.Is_hidden() {
                    Desk.Event_handler().await;
                }
            }

            Task::Manager_type::Sleep(Duration::from_millis(20)).await;
        }

        Ok(())
    }
}
