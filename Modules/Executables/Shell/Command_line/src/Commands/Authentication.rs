use crate::{Result_type, Shell_type};

impl Shell_type {
    pub fn Authenticate(&mut self) -> Result_type<String> {
        self.Standard.Print("Username: ");
        self.Standard.Out_flush();

        let mut Username = String::new();
        self.Standard.Read_line(&mut Username);

        self.Standard.Print("Password: ");
        self.Standard.Out_flush();

        let mut Password = String::new();
        self.Standard.Read_line(&mut Password);

        Ok(Username)
    }
}
