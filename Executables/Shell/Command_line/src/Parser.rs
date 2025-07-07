use alloc::vec::Vec;

use crate::{
    Error::{Error_type, Result_type},
    Tokenizer::Token_type,
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

    pub fn Get_arguments(&self) -> &[&str] {
        &self.arguments
    }
}

impl<'a> TryFrom<&[Token_type<'a>]> for Command_type<'a> {
    type Error = Error_type;

    fn try_from(Value: &[Token_type<'a>]) -> Result<Self, Self::Error> {
        let mut iterator = Value.iter();

        let Command = match iterator.next() {
            Some(Token_type::String(command)) => *command,
            _ => return Err(Error_type::Missing_command),
        };

        let mut Arguments = Vec::new();

        while let Some(Token_type::String(Argument)) = iterator.next() {
            Arguments.push(*Argument);
        }

        Ok(Self {
            command: Command,
            arguments: Arguments,
        })
    }
}

pub fn Parse(Tokens: Vec<Token_type<'_>>) -> Result_type<Vec<Command_type<'_>>> {
    let tokens = Tokens.clone();
    let split = tokens.split(|token| *token == Token_type::Pipe);

    let Commands = split
        .map(Command_type::try_from)
        .collect::<Result<Vec<Command_type>, Error_type>>()?;

    Ok(Commands)
}

#[cfg(test)]
mod Tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn Test_parse() {
        let Tokens = vec![
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String("main"),
        ];

        let Commands = Parse(Tokens).unwrap();

        assert_eq!(Commands.len(), 2);

        assert_eq!(Commands[0].command, "ls");
        assert_eq!(Commands[0].arguments, vec!["-l"]);

        assert_eq!(Commands[1].command, "grep");
        assert_eq!(Commands[1].arguments, vec!["main"]);

        let Tokens = vec![
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::String("-a"),
            Token_type::String("-h"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String("main"),
        ];

        let Commands = Parse(Tokens).unwrap();

        assert_eq!(Commands.len(), 2);

        assert_eq!(Commands[0].command, "ls");
        assert_eq!(Commands[0].arguments, vec!["-l", "-a", "-h"]);

        assert_eq!(Commands[1].command, "grep");
        assert_eq!(Commands[1].arguments, vec!["main"]);
    }
}
