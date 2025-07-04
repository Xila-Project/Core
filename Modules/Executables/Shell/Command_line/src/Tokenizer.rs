use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum Redirect_type_type {
    Output,
    Output_append,
    Input,
    Here_document,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect_type<'a> {
    pub Left: &'a str,
    pub Redirect_type: Redirect_type_type,
    pub Right: &'a str,
}

impl<'a> TryFrom<&'a str> for Redirect_type<'a> {
    type Error = ();

    fn try_from(Value: &'a str) -> Result<Self, Self::Error> {
        if let Some((Left, Right)) = Value.split_once(">>") {
            return Ok(Redirect_type {
                Left,
                Redirect_type: Redirect_type_type::Output_append,
                Right,
            });
        } else if let Some((Left, Right)) = Value.split_once("<<") {
            return Ok(Redirect_type {
                Left,
                Redirect_type: Redirect_type_type::Here_document,
                Right,
            });
        } else if let Some((Left, Right)) = Value.split_once(">") {
            return Ok(Redirect_type {
                Left,
                Redirect_type: Redirect_type_type::Output,
                Right,
            });
        } else if let Some((Left, Right)) = Value.split_once("<") {
            return Ok(Redirect_type {
                Left,
                Redirect_type: Redirect_type_type::Input,
                Right,
            });
        }

        Err(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token_type<'a> {
    String(&'a str),
    Pipe,
    Redirect(Redirect_type<'a>),
}

impl<'a> From<&'a str> for Token_type<'a> {
    fn from(Value: &'a str) -> Self {
        match Value {
            "|" => Token_type::Pipe,
            Value => {
                if let Ok(Redirect) = Redirect_type::try_from(Value) {
                    Token_type::Redirect(Redirect)
                } else {
                    Token_type::String(Value)
                }
            }
        }
    }
}

pub fn Tokenize<'a>(Input: &'a [&'a str]) -> Vec<Token_type<'a>> {
    Input.iter().map(|Value| Token_type::from(*Value)).collect()
}

#[cfg(test)]
mod Tests {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn Test_tokenize_empty() {
        let Input = "".split_whitespace().collect::<Vec<&str>>();

        let Expected: Vec<Token_type> = Vec::new();

        assert_eq!(Tokenize(&Input), Expected);
    }

    #[test]
    fn Test_tokenize_complex() {
        let Input = "ls -l | grep .rs 2>&1 > output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String(".rs"),
            Token_type::Redirect(Redirect_type {
                Left: "2",
                Redirect_type: Redirect_type_type::Output,
                Right: "&1",
            }),
            Token_type::Redirect(Redirect_type {
                Left: "",
                Redirect_type: Redirect_type_type::Output,
                Right: "",
            }),
            Token_type::String("output.txt"),
        ];
        assert_eq!(Tokenize(&Input), Expected);
    }

    #[test]
    fn Test_tokenize_pipe() {
        let Input = "ls -l | grep .rs".split_whitespace().collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String(".rs"),
        ];

        assert_eq!(Tokenize(&Input), &Expected);
    }

    #[test]
    fn Test_tokenize_redirect() {
        let Input = "ls -l > output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                Left: "",
                Redirect_type: Redirect_type_type::Output,
                Right: "",
            }),
            Token_type::String("output.txt"),
        ];

        assert_eq!(Tokenize(&Input), &Expected);

        let Input = "ls -l < input.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                Left: "",
                Redirect_type: Redirect_type_type::Input,
                Right: "",
            }),
            Token_type::String("input.txt"),
        ];

        assert_eq!(Tokenize(&Input), &Expected);

        let Input = "ls -l >> output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                Left: "",
                Redirect_type: Redirect_type_type::Output_append,
                Right: "",
            }),
            Token_type::String("output.txt"),
        ];

        assert_eq!(Tokenize(&Input), &Expected);

        let Input = "ls -l << EOF".split_whitespace().collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                Left: "",
                Redirect_type: Redirect_type_type::Here_document,
                Right: "",
            }),
            Token_type::String("EOF"),
        ];

        assert_eq!(Tokenize(&Input), &Expected);

        let Input = "ls -l 2>&1".split_whitespace().collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                Left: "2",
                Redirect_type: Redirect_type_type::Output,
                Right: "&1",
            }),
        ];

        assert_eq!(Tokenize(&Input), &Expected);

        let Input = "ls -l 2> output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let Expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                Left: "2",
                Redirect_type: Redirect_type_type::Output,
                Right: "",
            }),
            Token_type::String("output.txt"),
        ];

        assert_eq!(Tokenize(&Input), &Expected);
    }
}
