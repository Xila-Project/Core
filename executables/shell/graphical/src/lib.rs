#![no_std]

mod desk;
mod error;
mod home;
mod icon;
mod layout;
mod login;
mod shortcut;

extern crate alloc;

use crate::{desk::Desk, error::Error};
use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt::Write;
use core::num::NonZeroUsize;
use core::time::Duration;
use getargs::Arg;
use home::Home;
use layout::Layout;
use login::Login;
use xila::executable::{self, ExecutableTrait, Standard};
use xila::task;
use xila::users;

pub async fn main(mut standard: Standard, arguments: Vec<String>) -> Result<(), NonZeroUsize> {
    let arguments = arguments.iter().map(|s| s.as_str());

    let mut options = getargs::Options::new(arguments);

    let mut show_keyboard = false;

    while let Some(argument) = options.next_arg().map_err(|e| {
        writeln!(standard.error(), "{}", e).ok();
        NonZeroUsize::new(1).unwrap()
    })? {
        match argument {
            Arg::Short('k') | Arg::Long("show-keyboard") => {
                show_keyboard = true;
            }
            _ => {}
        }
    }

    Shell::new(standard, show_keyboard).await.main().await
}

pub struct Shell {
    _standard: Standard,
    running: bool,
    layout: Layout,
    desk: Option<Box<Desk>>,
    _home: Option<Box<Home>>,
    login: Option<Box<Login>>,
}

pub struct ShellExecutable;

impl ExecutableTrait for ShellExecutable {
    fn main(standard: Standard, arguments: Vec<String>) -> executable::MainFuture {
        Box::pin(async move { main(standard, arguments).await })
    }
}

impl Shell {
    pub async fn new(standard: Standard, show_keyboard: bool) -> Self {
        let layout = Layout::new(show_keyboard).await.unwrap();

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

    pub async fn main(&mut self) -> Result<(), NonZeroUsize> {
        while self.running {
            self.layout.run().await;

            if let Some(login) = &mut self.login {
                login.event_handler().await;

                if let Some(user) = login.get_logged_user() {
                    let user_name = users::get_instance().get_user_name(user).await.unwrap();

                    let task = task::get_instance().get_current_task_identifier().await;

                    task::get_instance()
                        .set_environment_variable(task, "User", user_name.as_str())
                        .await
                        .map_err(Error::FailedToSetEnvironmentVariable)?;

                    self.desk = Some(Box::new(Desk::new(self.layout.get_windows_parent()).await?));

                    if let Some(desk) = &mut self.desk {
                        self._home = Some(Box::new(Home::new(desk.get_window_object()).await?));
                    }

                    self.login = None;
                }
            }

            if let Some(desk) = &mut self.desk
                && !desk.is_hidden()
            {
                desk.handle_events().await;
            }

            task::sleep(Duration::from_millis(50)).await;
        }

        Ok(())
    }
}
