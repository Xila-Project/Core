use alloc::string::String;

use crate::{Result, Shell, error::Error};

impl Shell {
    pub async fn authenticate(&mut self) -> Result<String> {
        self.standard.print("Username: ").await;
        self.standard.out_flush().await;

        let mut user_name = String::new();
        self.standard.read_line(&mut user_name).await;

        self.standard.print("Password: ").await;
        self.standard.out_flush().await;

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
