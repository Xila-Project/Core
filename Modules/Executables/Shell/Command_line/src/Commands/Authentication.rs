use crate::{Error::Error_type, Result_type, Shell_type};

impl Shell_type {
    pub fn Authenticate(&mut self) -> Result_type<String> {
        self.Standard.Print("Username: ");
        self.Standard.Out_flush();

        let mut User_name = String::new();
        self.Standard.Read_line(&mut User_name);

        self.Standard.Print("Password: ");
        self.Standard.Out_flush();

        let mut Password = String::new();
        self.Standard.Read_line(&mut Password);

        // - Check the user name and the password
        let User_identifier = Authentication::Authenticate_user(
            Virtual_file_system::Get_instance(),
            &User_name,
            &Password,
        )
        .map_err(Error_type::Authentication_failed)?;

        // - Set the user
        let Task_manager = Task::Get_instance();

        let Task = Task_manager
            .Get_current_task_identifier()
            .map_err(Error_type::Failed_to_set_task_user)?;

        Task_manager
            .Set_user(Task, User_identifier)
            .map_err(Error_type::Failed_to_set_task_user)?;

        Task_manager
            .Set_environment_variable(Task, "User", &User_name)
            .map_err(Error_type::Failed_to_set_environment_variable)?;

        Ok(User_name)
    }
}
