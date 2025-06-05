use crate::Shell_type;

impl Shell_type {
    pub async fn Echo(&mut self, Arguments: &[&str]) {
        for Argument in Arguments {
            self.Standard.Print(Argument).await;
            self.Standard.Print(" ").await;
        }
        self.Standard.Print("\n").await;
    }
}
