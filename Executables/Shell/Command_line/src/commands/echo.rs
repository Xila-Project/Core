use crate::Shell_type;

impl Shell_type {
    pub async fn echo(&mut self, arguments: &[&str]) {
        for argument in arguments {
            self.standard.print(argument).await;
            self.standard.print(" ").await;
        }
        self.standard.print("\n").await;
    }
}
