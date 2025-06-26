use crate::Shell_type;

impl Shell_type {
    pub async fn Exit(&mut self, Arguments: &[&str]) {
        if !Arguments.is_empty() {
            self.Standard
                .Print_error_line("Invalid number of arguments")
                .await;
        }

        self.Running = false;
    }
}
