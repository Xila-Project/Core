#![no_std]
#![allow(non_camel_case_types)]

mod desk;
mod error;
mod home;
mod icon;
mod layout;
mod login;
mod shortcut;

extern crate alloc;

use ::executable::Standard_type;
use alloc::{boxed::Box, string::String, vec::Vec};
use core::num::NonZeroUsize;
use core::time::Duration;
use file_system::Path_type;
use home::Home_type;
use layout::Layout_type;
use login::Login_type;

use crate::{desk::Desk_type, error::Error_type, shortcut::Shortcut_type};

pub async fn main(standard: Standard_type, arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::new(standard).await.main(arguments).await
}

pub struct Shell_type {
    _standard: Standard_type,
    running: bool,
    layout: Layout_type,
    desk: Option<Box<Desk_type>>,
    _home: Option<Box<Home_type>>,
    login: Option<Box<Login_type>>,
}

pub struct Shell_executable_type;

executable::Implement_executable_device!(
    Structure: Shell_executable_type,
    Mount_path: "/Binaries/Graphical_shell",
    Main_function: main,
);

impl Shell_type {
    pub async fn new(standard: Standard_type) -> Self {
        let layout = Layout_type::new().await.unwrap();

        let login = Box::new(Login_type::new().await.unwrap());

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
                return Err(Error_type::Missing_arguments.into());
            }

            Shortcut_type::add(Path_type::from_str(arguments[1])).await?;
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
                        .map_err(Error_type::Failed_to_set_environment_variable)?;

                    self.desk = Some(Box::new(
                        Desk_type::new(self.layout.get_windows_parent()).await?,
                    ));

                    if let Some(desk) = &mut self.desk {
                        self._home =
                            Some(Box::new(Home_type::new(desk.get_window_object()).await?));
                    }

                    self.login = None;
                }
            }

            if let Some(desk) = &mut self.desk {
                if !desk.is_hidden() {
                    desk.event_handler().await;
                }
            }

            task::Manager_type::sleep(Duration::from_millis(20)).await;
        }

        Ok(())
    }
}
