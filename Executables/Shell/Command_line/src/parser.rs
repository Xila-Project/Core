use alloc::vec::Vec;

use crate::{
    error::{Error, Result},
    tokenizer::Token,
};

#[derive(Debug, Clone)]
pub struct Command<'a> {
    command: &'a str,
    arguments: Vec<&'a str>,
}

impl Command<'_> {
    pub fn get_command(&self) -> &str {
        self.command
    }

    pub fn get_arguments(&self) -> &[&str] {
        &self.arguments
    }
}

impl<'a> TryFrom<&[Token<'a>]> for Command<'a> {
    type Error = Error;

    fn try_from(value: &[Token<'a>]) -> Result<Self> {
        let mut iterator = value.iter();

        let command = match iterator.next() {
            Some(Token::String(command)) => *command,
            _ => return Err(Error::MissingCommand),
        };

        let mut arguments = Vec::new();

        while let Some(Token::String(argument)) = iterator.next() {
            arguments.push(*argument);
        }

        Ok(Self { command, arguments })
    }
}

pub fn parse(tokens: Vec<Token<'_>>) -> Result<Vec<Command<'_>>> {
    let tokens = tokens.clone();
    let split = tokens.split(|token| *token == Token::Pipe);

    let commands = split
        .map(Command::try_from)
        .collect::<core::result::Result<Vec<Command>, Error>>()?;

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_parse() {
        let tokens = vec![
            Token::String("ls"),
            Token::String("-l"),
            Token::Pipe,
            Token::String("grep"),
            Token::String("main"),
        ];

        let commands = parse(tokens).unwrap();

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].command, "ls");
        assert_eq!(commands[0].arguments, vec!["-l"]);

        assert_eq!(commands[1].command, "grep");
        assert_eq!(commands[1].arguments, vec!["main"]);

        let tokens = vec![
            Token::String("ls"),
            Token::String("-l"),
            Token::String("-a"),
            Token::String("-h"),
            Token::Pipe,
            Token::String("grep"),
            Token::String("main"),
        ];

        let commands = parse(tokens).unwrap();

        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0].command, "ls");
        assert_eq!(commands[0].arguments, vec!["-l", "-a", "-h"]);

        assert_eq!(commands[1].command, "grep");
        assert_eq!(commands[1].arguments, vec!["main"]);
    }
}
