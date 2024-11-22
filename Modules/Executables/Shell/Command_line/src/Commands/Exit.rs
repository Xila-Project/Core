use crate::Shell_type;

impl Shell_type {
    pub fn Exit(&mut self, Arguments: &[&str]) {
        if !Arguments.is_empty() {
            self.Standard
                .Print_error_line("Invalid number of arguments");
        }

        self.Running = false;
    }
}
