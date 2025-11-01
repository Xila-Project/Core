use crate::{Result, Shell, error::Error};
use alloc::string::String;
use core::fmt::Write;
use xila::{authentication, internationalization::translate, task, virtual_file_system};

impl Shell {
    pub async fn authenticate(&mut self) -> Result<String> {
        write!(self.standard.out(), translate!("User name: "))?;
        let _ = self.standard.out().flush().await;

        let mut user_name = String::new();
        self.standard.read_line(&mut user_name).await;

        write!(self.standard.out(), translate!("Password: "))?;
        let _ = self.standard.out().flush().await;

        let mut password = String::new();
        self.standard.read_line(&mut password).await;

        // - Check the user name and the password
        let user_identifier = authentication::authenticate_user(
            virtual_file_system::get_instance(),
            &user_name,
            &password,
        )
        .await
        .map_err(Error::AuthenticationFailed)?;

        // - Set the user
        let task_manager = task::get_instance();

        let task = task_manager.get_current_task_identifier().await;

        task_manager
            .set_user(task, user_identifier)
            .await
            .map_err(Error::FailedToSetTaskUser)?;

        task_manager
            .set_environment_variable(task, "User", &user_name)
            .await
            .map_err(Error::FailedToSetEnvironmentVariable)?;

        Ok(user_name)
    }
}
