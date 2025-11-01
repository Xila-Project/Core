use crate::{Result, Shell};
use core::fmt::Write;

impl Shell {
    pub async fn clear(&mut self, _: &[&str]) -> Result<()> {
        write!(self.standard.out(), "\x1B[2J")?;
        write!(self.standard.out(), "\x1B[H")?;

        Ok(())
    }
}
