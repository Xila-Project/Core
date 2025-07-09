use alloc::vec::Vec;

use crate::{
    error::{Error_type, Result_type},
    tokenizer::Token_type,
};

#[derive(Debug, Clone)]
pub struct Command_type<'a> {
    command: &'a str,
    arguments: Vec<&'a str>,
}

impl Command_type<'_> {
    pub fn get_command(&self) -> &str {
        self.command
    }

    pub fn get_arguments(&self) -> &[&str] {
        &self.arguments
    }
}

impl<'a> TryFrom<&[Token_type<'a>]> for Command_type<'a> {
    type Error = Error_type;

    fn try_from(value: &[Token_type<'a>]) -> Result<Self, Self::Error> {
        let mut iterator = value.iter();

        let command = match iterator.next() {
            Some(Token_type::String(command)) => *command,
            _ => return Err(Error_type::Missing_command),
        };

        let mut arguments = Vec::new();

        while let Some(Token_type::String(argument)) = iterator.next() {
            arguments.push(*argument);
        }

        Ok(Self { command, arguments })
    }
}

pub fn parse(tokens: Vec<Token_type<'_>>) -> Result_type<Vec<Command_type<'_>>> {
    let tokens = tokens.clone();
    let split = tokens.split(|token| *token == Token_type::Pipe);

    let commands = split
        .map(Command_type::try_from)
        .collect::<Result<Vec<Command_type>, Error_type>>()?;

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String("main"),
        ];

        let commands = parse(tokens).unwrap();

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].command, "ls");
        assert_eq!(commands[0].arguments, vec!["-l"]);

        assert_eq!(commands[1].command, "grep");
        assert_eq!(commands[1].arguments, vec!["main"]);

        let tokens = vec![
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::String("-a"),
            Token_type::String("-h"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String("main"),
        ];

        let commands = parse(tokens).unwrap();

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].command, "ls");
        assert_eq!(commands[0].arguments, vec!["-l", "-a", "-h"]);

        assert_eq!(commands[1].command, "grep");
        assert_eq!(commands[1].arguments, vec!["main"]);
    }
}
