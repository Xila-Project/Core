use crate::{Result, Shell};

impl Shell {
    pub async fn echo(&mut self, arguments: &[&str]) -> Result<()> {
        for argument in arguments {
            self.standard.print(argument).await;
            self.standard.print(" ").await;
        }
        self.standard.print("\n").await;

        Ok(())
    }
}
