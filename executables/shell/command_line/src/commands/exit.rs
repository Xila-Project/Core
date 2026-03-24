use executable_macros::GetArgs;
use getargs::Options;

use crate::{Result, Shell};

#[derive(GetArgs)]
struct ExitArguments {}

impl Shell {
    pub async fn exit<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let _ = ExitArguments::parse(options)?;

        self.running = false;

        Ok(())
    }
}
