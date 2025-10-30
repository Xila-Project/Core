use crate::{Result, Shell};

impl Shell {
    pub async fn clear(&mut self, _: &[&str]) -> Result<()> {
        self.standard.print("\x1B[2J").await;
        self.standard.print("\x1B[H").await;

        Ok(())
    }
}
