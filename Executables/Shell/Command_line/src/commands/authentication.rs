use alloc::string::String;

use crate::{error::Error_type, Result_type, Shell_type};

impl Shell_type {
    pub async fn authenticate(&mut self) -> Result_type<String> {
        self.standard.print("Username: ").await;
        self.standard.out_flush().await;

        let mut user_name = String::new();
        self.standard.read_line(&mut user_name).await;

        self.standard.print("Password: ").await;
        self.standard.out_flush().await;

        let mut password = String::new();
        self.standard.read_line(&mut password).await;

        // - Check the user name and the password
        let User_identifier = authentication::authenticate_user(
            virtual_file_system::get_instance(),
            &user_name,
            &password,
        )
        .await
        .map_err(Error_type::Authentication_failed)?;

        // - Set the user
        let task_manager = task::get_instance();

        let task = task_manager.get_current_task_identifier().await;

        task_manager
            .set_user(task, User_identifier)
            .await
            .map_err(Error_type::Failed_to_set_task_user)?;

        task_manager
            .Set_environment_variable(task, "User", &user_name)
            .await
            .map_err(Error_type::Failed_to_set_environment_variable)?;

        Ok(user_name)
    }
}
