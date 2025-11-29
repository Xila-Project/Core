use crate::{Result, Shell, commands::check_no_more_arguments};
use core::fmt::Write;
use getargs::Options;

impl Shell {
    pub async fn clear<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        check_no_more_arguments(options)?;

        write!(self.standard.out(), "\x1B[2J")?;
        write!(self.standard.out(), "\x1B[H")?;

        Ok(())
    }
}
