use crate::Shell_type;

impl Shell_type {
    pub async fn echo(&mut self, Arguments: &[&str]) {
        for Argument in Arguments {
            self.standard.Print(Argument).await;
            self.standard.Print(" ").await;
        }
        self.standard.Print("\n").await;
    }
}
