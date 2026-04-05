use crate::Shell;

use core::fmt::Write;

impl Shell {
    pub async fn print_working_directory<'a, I>(
        &mut self,
        _: &mut getargs::Options<&'a str, I>,
    ) -> crate::Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        writeln!(self.standard.out(), "{}", self.current_directory)?;
        Ok(())
    }
}
