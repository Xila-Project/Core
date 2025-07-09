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
    pub left: &'a str,
    pub redirect_type: Redirect_type_type,
    pub right: &'a str,
}

impl<'a> TryFrom<&'a str> for Redirect_type<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some((left, right)) = value.split_once(">>") {
            return Ok(Redirect_type {
                left,
                redirect_type: Redirect_type_type::Output_append,
                right,
            });
        } else if let Some((left, right)) = value.split_once("<<") {
            return Ok(Redirect_type {
                left,
                redirect_type: Redirect_type_type::Here_document,
                right,
            });
        } else if let Some((left, right)) = value.split_once(">") {
            return Ok(Redirect_type {
                left,
                redirect_type: Redirect_type_type::Output,
                right,
            });
        } else if let Some((left, right)) = value.split_once("<") {
            return Ok(Redirect_type {
                left,
                redirect_type: Redirect_type_type::Input,
                right,
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
    fn from(value: &'a str) -> Self {
        match value {
            "|" => Token_type::Pipe,
            value => {
                if let Ok(redirect) = Redirect_type::try_from(value) {
                    Token_type::Redirect(redirect)
                } else {
                    Token_type::String(value)
                }
            }
        }
    }
}

pub fn tokenize<'a>(input: &'a [&'a str]) -> Vec<Token_type<'a>> {
    input.iter().map(|value| Token_type::from(*value)).collect()
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let input = "".split_whitespace().collect::<Vec<&str>>();

        let expected: Vec<Token_type> = Vec::new();

        assert_eq!(tokenize(&input), expected);
    }

    #[test]
    fn test_tokenize_complex() {
        let input = "ls -l | grep .rs 2>&1 > output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String(".rs"),
            Token_type::Redirect(Redirect_type {
                left: "2",
                redirect_type: Redirect_type_type::Output,
                right: "&1",
            }),
            Token_type::Redirect(Redirect_type {
                left: "",
                redirect_type: Redirect_type_type::Output,
                right: "",
            }),
            Token_type::String("output.txt"),
        ];
        assert_eq!(tokenize(&input), expected);
    }

    #[test]
    fn test_tokenize_pipe() {
        let input = "ls -l | grep .rs".split_whitespace().collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Pipe,
            Token_type::String("grep"),
            Token_type::String(".rs"),
        ];

        assert_eq!(tokenize(&input), &expected);
    }

    #[test]
    fn test_tokenize_redirect() {
        let input = "ls -l > output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                left: "",
                redirect_type: Redirect_type_type::Output,
                right: "",
            }),
            Token_type::String("output.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l < input.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                left: "",
                redirect_type: Redirect_type_type::Input,
                right: "",
            }),
            Token_type::String("input.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l >> output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                left: "",
                redirect_type: Redirect_type_type::Output_append,
                right: "",
            }),
            Token_type::String("output.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l << EOF".split_whitespace().collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                left: "",
                redirect_type: Redirect_type_type::Here_document,
                right: "",
            }),
            Token_type::String("EOF"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l 2>&1".split_whitespace().collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                left: "2",
                redirect_type: Redirect_type_type::Output,
                right: "&1",
            }),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l 2> output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token_type::String("ls"),
            Token_type::String("-l"),
            Token_type::Redirect(Redirect_type {
                left: "2",
                redirect_type: Redirect_type_type::Output,
                right: "",
            }),
            Token_type::String("output.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);
    }
}
