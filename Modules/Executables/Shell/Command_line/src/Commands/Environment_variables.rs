use crate::Shell_type;

impl Shell_type {
    pub fn Set_environment_variable(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments");
            return;
        }

        let (Name, Value) = match Arguments[0].split_once('=') {
            Some((Name, Value)) => (Name, Value),
            None => {
                self.Standard.Print_error_line("Invalid argument");
                return;
            }
        };

        if let Err(Error) =
            Task::Get_instance().Set_environment_variable(self.Standard.Get_task(), Name, Value)
        {
            self.Standard
                .Print_error_line(&format!("Failed to set environment variable: {}", Error));
        }
    }

    pub fn Remove_environment_variable(&mut self, Arguments: &[&str]) {
        if Arguments.len() != 1 {
            self.Standard
                .Print_error_line("Invalid number of arguments");
            return;
        }

        let Name = Arguments[0];

        if let Err(Error) =
            Task::Get_instance().Remove_environment_variable(self.Standard.Get_task(), Name)
        {
            self.Standard
                .Print_error_line(&format!("Failed to unset environment variable: {}", Error));
        }
    }
}
