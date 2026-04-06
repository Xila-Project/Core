use executable_macros::GetArgs;
use getargs::Options;
use xila::file_system::Path;

use crate::Result;

use super::{CommandContext, UserCommand};

pub struct ExitCommand;

impl UserCommand for ExitCommand {
    async fn execute<'a, I, C>(
        &self,
        context: &mut C,
        options: &mut Options<&'a str, I>,
        _paths: &[&Path],
    ) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
        C: CommandContext,
    {
        execute_exit(context, options)
    }
}

#[derive(GetArgs)]
struct ExitArguments {}

fn execute_exit<'a, I, C>(context: &mut C, options: &mut Options<&'a str, I>) -> Result<()>
where
    I: Iterator<Item = &'a str>,
    C: CommandContext,
{
    let _ = ExitArguments::parse(options)?;

    context.stop();

    Ok(())
}
