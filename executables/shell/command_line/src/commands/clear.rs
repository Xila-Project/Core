use crate::{Result, Shell};
use core::fmt::Write;
use executable_macros::GetArgs;
use getargs::Options;

#[derive(GetArgs)]
struct ClearArguments {}

impl Shell {
    pub async fn clear<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let _ = ClearArguments::parse(options)?;

        write!(self.standard.out(), "\x1B[2J")?;
        write!(self.standard.out(), "\x1B[H")?;

        Ok(())
    }
}
