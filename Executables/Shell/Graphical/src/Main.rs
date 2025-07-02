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
    Shell_type::New(Standard).await.Main(Arguments).await
}

impl Shell_type {
    pub async fn New(Standard: Standard_type) -> Self {
        let Layout = Layout_type::New().await.unwrap();

        let Login = Box::new(Login_type::New().await.unwrap());

        Self {
            _Standard: Standard,
            Layout,
            Desk: None,
            Running: true,
            _Home: None,
            Login: Some(Login),
        }
    }

    pub async fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        let Arguments: Vec<&str> = Arguments.split_whitespace().collect();

        if Arguments.first() == Some(&"add_shortcut") {
            if Arguments.len() != 2 {
                return Err(Error_type::Missing_arguments.into());
            }

            Shortcut_type::Add(Path_type::From_str(Arguments[1])).await?;
        }

        while self.Running {
            self.Layout.Loop().await;

            if let Some(Login) = &mut self.Login {
                Login.Event_handler().await;

                if let Some(User) = Login.Get_logged_user() {
                    let User_name = Users::Get_instance().Get_user_name(User).await.unwrap();

                    Task::Get_instance()
                        .Set_environment_variable(
                            self._Standard.Get_task(),
                            "User",
                            User_name.as_str(),
                        )
                        .await
                        .map_err(Error_type::Failed_to_set_environment_variable)?;

                    self.Desk = Some(Box::new(
                        Desk_type::New(self.Layout.Get_windows_parent()).await?,
                    ));

                    if let Some(Desk) = &mut self.Desk {
                        self._Home =
                            Some(Box::new(Home_type::New(Desk.Get_window_object()).await?));
                    }

                    self.Login = None;
                }
            }

            if let Some(Desk) = &mut self.Desk {
                if !Desk.Is_hidden() {
                    Desk.Event_handler().await;
                }
            }

            Task::Manager_type::Sleep(Duration::from_millis(20)).await;
        }

        Ok(())
    }
}
