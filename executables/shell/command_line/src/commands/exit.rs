use crate::{Error, Result, Shell};

impl Shell {
    pub async fn exit(&mut self, arguments: &[&str]) -> Result<()> {
        if !arguments.is_empty() {
            return Err(Error::InvalidNumberOfArguments);
        }

        self.running = false;

        Ok(())
    }
}
