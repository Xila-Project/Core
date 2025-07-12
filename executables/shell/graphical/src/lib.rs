#![no_std]

mod desk;
mod error;
mod home;
mod icon;
mod layout;
mod login;
mod shortcut;

extern crate alloc;

use ::executable::Standard;
use alloc::{boxed::Box, string::String, vec::Vec};
use core::num::NonZeroUsize;
use core::time::Duration;
use file_system::Path;
use home::HomeType;
use layout::Layout;
use login::Login;

use crate::{desk::DeskType, error::Error, shortcut::ShortcutType};

pub async fn main(standard: Standard, arguments: String) -> Result<(), NonZeroUsize> {
    ShellType::new(standard).await.main(arguments).await
}

pub struct ShellType {
    _standard: Standard,
    running: bool,
    layout: Layout,
    desk: Option<Box<DeskType>>,
    _home: Option<Box<HomeType>>,
    login: Option<Box<Login>>,
}

pub struct ShellExecutableType;

executable::implement_executable_device!(
    Structure: ShellExecutableType,
    Mount_path: "/Binaries/Graphical_shell",
    Main_function: main,
);

impl ShellType {
    pub async fn new(standard: Standard) -> Self {
        let layout = Layout::new().await.unwrap();

        let login = Box::new(Login::new().await.unwrap());

        Self {
            _standard: standard,
            layout,
            desk: None,
            running: true,
            _home: None,
            login: Some(login),
        }
    }

    pub async fn main(&mut self, arguments: String) -> Result<(), NonZeroUsize> {
        let arguments: Vec<&str> = arguments.split_whitespace().collect();

        if arguments.first() == Some(&"add_shortcut") {
            if arguments.len() != 2 {
                return Err(Error::MissingArguments.into());
            }

            ShortcutType::add(Path::from_str(arguments[1])).await?;
        }

        while self.running {
            self.layout.r#loop().await;

            if let Some(login) = &mut self.login {
                login.event_handler().await;

                if let Some(user) = login.get_logged_user() {
                    let user_name = users::get_instance().get_user_name(user).await.unwrap();

                    task::get_instance()
                        .set_environment_variable(
                            self._standard.get_task(),
                            "User",
                            user_name.as_str(),
                        )
                        .await
                        .map_err(Error::FailedToSetEnvironmentVariable)?;

                    self.desk = Some(Box::new(
                        DeskType::new(self.layout.get_windows_parent()).await?,
                    ));

                    if let Some(desk) = &mut self.desk {
                        self._home = Some(Box::new(HomeType::new(desk.get_window_object()).await?));
                    }

                    self.login = None;
                }
            }

            if let Some(desk) = &mut self.desk {
                if !desk.is_hidden() {
                    desk.event_handler().await;
                }
            }

            task::Manager::sleep(Duration::from_millis(20)).await;
        }

        Ok(())
    }
}
