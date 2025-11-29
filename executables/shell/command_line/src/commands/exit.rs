use getargs::Options;

use crate::{Result, Shell, commands::check_no_more_arguments};

impl Shell {
    pub async fn exit<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        check_no_more_arguments(options)?;

        self.running = false;

        Ok(())
    }
}
