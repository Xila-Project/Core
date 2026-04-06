use crate::Result;
use executable_macros::GetArgs;
use getargs::Options;
use xila::file_system::Path;

use super::{CommandContext, UserCommand};

pub struct ClearCommand;

impl UserCommand for ClearCommand {
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
        execute_clear(context, options)
    }
}

#[derive(GetArgs)]
struct ClearArguments {}

fn execute_clear<'a, I, C>(context: &mut C, options: &mut Options<&'a str, I>) -> Result<()>
where
    I: Iterator<Item = &'a str>,
    C: CommandContext,
{
    let _ = ClearArguments::parse(options)?;

    context.write_out_fmt(format_args!("\x1B[2J"))?;
    context.write_out_fmt(format_args!("\x1B[H"))?;

    Ok(())
}
