use crate::{
    Error::{Error_type, Result_type},
    Tokenizer::Token_type,
};

#[derive(Debug, Clone)]
pub struct Command_type<'a> {
    Command: &'a str,
    Arguments: Vec<&'a str>,
}

impl Command_type<'_> {
    pub fn Get_command(&self) -> &str {
        self.Command
    }

    pub fn Get_arguments(&self) -> &[&str] {
        &self.Arguments
    }
}

impl<'a> TryFrom<&[Token_type<'a>]> for Command_type<'a> {
    type Error = Error_type;

    fn try_from(Value: &[Token_type<'a>]) -> Result<Self, Self::Error> {
        let mut Iterator = Value.iter();

        let Command = match Iterator.next() {
            Some(Token_type::String(Command)) => *Command,
            _ => return Err(Error_type::Missing_command),
        };

        let mut Arguments = Vec::new();

        while let Some(Token_type::String(Argument)) = Iterator.next() {
            Arguments.push(*Argument);
        }

        Ok(Self { Command, Arguments })
    }
}

pub fn Parse(Tokens: Vec<Token_type<'_>>) -> Result_type<Vec<Command_type<'_>>> {
    let Tokens = Tokens.clone();
    let Split = Tokens.split(|Token| *Token == Token_type::Pipe);

    let Commands = Split
        .map(Command_type::try_from)
        .collect::<Result<Vec<Command_type>, Error_type>>()?;

    Ok(Commands)
}

#[cfg(test)]
mod Tests {
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

        assert_eq!(Commands[0].Command, "ls");
        assert_eq!(Commands[0].Arguments, vec!["-l"]);

        assert_eq!(Commands[1].Command, "grep");
        assert_eq!(Commands[1].Arguments, vec!["main"]);

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

        assert_eq!(Commands[0].Command, "ls");
        assert_eq!(Commands[0].Arguments, vec!["-l", "-a", "-h"]);

        assert_eq!(Commands[1].Command, "grep");
        assert_eq!(Commands[1].Arguments, vec!["main"]);
    }
}
