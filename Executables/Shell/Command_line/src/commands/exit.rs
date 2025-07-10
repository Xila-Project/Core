use crate::Shell;

impl Shell {
    pub async fn exit(&mut self, arguments: &[&str]) {
        if !arguments.is_empty() {
            self.standard
                .print_error_line("Invalid number of arguments")
                .await;
        }

        self.running = false;
    }
}
