use alloc::string::String;

use crate::{Error::Error_type, Result_type, Shell_type};

impl Shell_type {
    pub async fn Authenticate(&mut self) -> Result_type<String> {
        self.Standard.Print("Username: ").await;
        self.Standard.Out_flush().await;

        let mut User_name = String::new();
        self.Standard.Read_line(&mut User_name).await;

        self.Standard.Print("Password: ").await;
        self.Standard.Out_flush().await;

        let mut Password = String::new();
        self.Standard.Read_line(&mut Password).await;

        // - Check the user name and the password
        let User_identifier = Authentication::Authenticate_user(
            Virtual_file_system::Get_instance(),
            &User_name,
            &Password,
        )
        .await
        .map_err(Error_type::Authentication_failed)?;

        // - Set the user
        let Task_manager = Task::Get_instance();

        let Task = Task_manager.Get_current_task_identifier().await;

        Task_manager
            .Set_user(Task, User_identifier)
            .await
            .map_err(Error_type::Failed_to_set_task_user)?;

        Task_manager
            .Set_environment_variable(Task, "User", &User_name)
            .await
            .map_err(Error_type::Failed_to_set_environment_variable)?;

        Ok(User_name)
    }
}
