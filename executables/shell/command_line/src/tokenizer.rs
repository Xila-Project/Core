use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectKind {
    Output,
    OutputAppend,
    Input,
    HereDocument,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect<'a> {
    pub left: &'a str,
    pub redirect_type: RedirectKind,
    pub right: &'a str,
}

impl<'a> TryFrom<&'a str> for Redirect<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some((left, right)) = value.split_once(">>") {
            return Ok(Redirect {
                left,
                redirect_type: RedirectKind::OutputAppend,
                right,
            });
        } else if let Some((left, right)) = value.split_once("<<") {
            return Ok(Redirect {
                left,
                redirect_type: RedirectKind::HereDocument,
                right,
            });
        } else if let Some((left, right)) = value.split_once(">") {
            return Ok(Redirect {
                left,
                redirect_type: RedirectKind::Output,
                right,
            });
        } else if let Some((left, right)) = value.split_once("<") {
            return Ok(Redirect {
                left,
                redirect_type: RedirectKind::Input,
                right,
            });
        }

        Err(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    String(&'a str),
    Pipe,
    Redirect(Redirect<'a>),
}

impl<'a> From<&'a str> for Token<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "|" => Token::Pipe,
            value => {
                if let Ok(redirect) = Redirect::try_from(value) {
                    Token::Redirect(redirect)
                } else {
                    Token::String(value)
                }
            }
        }
    }
}

pub fn tokenize<'a>(input: &'a [&'a str]) -> Vec<Token<'a>> {
    input.iter().map(|value| Token::from(*value)).collect()
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let input = "".split_whitespace().collect::<Vec<&str>>();

        let expected: Vec<Token> = Vec::new();

        assert_eq!(tokenize(&input), expected);
    }

    #[test]
    fn test_tokenize_complex() {
        let input = "ls -l | grep .rs 2>&1 > output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Pipe,
            Token::String("grep"),
            Token::String(".rs"),
            Token::Redirect(Redirect {
                left: "2",
                redirect_type: RedirectKind::Output,
                right: "&1",
            }),
            Token::Redirect(Redirect {
                left: "",
                redirect_type: RedirectKind::Output,
                right: "",
            }),
            Token::String("output.txt"),
        ];
        assert_eq!(tokenize(&input), expected);
    }

    #[test]
    fn test_tokenize_pipe() {
        let input = "ls -l | grep .rs".split_whitespace().collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Pipe,
            Token::String("grep"),
            Token::String(".rs"),
        ];

        assert_eq!(tokenize(&input), &expected);
    }

    #[test]
    fn test_tokenize_redirect() {
        let input = "ls -l > output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Redirect(Redirect {
                left: "",
                redirect_type: RedirectKind::Output,
                right: "",
            }),
            Token::String("output.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l < input.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Redirect(Redirect {
                left: "",
                redirect_type: RedirectKind::Input,
                right: "",
            }),
            Token::String("input.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l >> output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Redirect(Redirect {
                left: "",
                redirect_type: RedirectKind::OutputAppend,
                right: "",
            }),
            Token::String("output.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l << EOF".split_whitespace().collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Redirect(Redirect {
                left: "",
                redirect_type: RedirectKind::HereDocument,
                right: "",
            }),
            Token::String("EOF"),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l 2>&1".split_whitespace().collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Redirect(Redirect {
                left: "2",
                redirect_type: RedirectKind::Output,
                right: "&1",
            }),
        ];

        assert_eq!(tokenize(&input), &expected);

        let input = "ls -l 2> output.txt"
            .split_whitespace()
            .collect::<Vec<&str>>();

        let expected = [
            Token::String("ls"),
            Token::String("-l"),
            Token::Redirect(Redirect {
                left: "2",
                redirect_type: RedirectKind::Output,
                right: "",
            }),
            Token::String("output.txt"),
        ];

        assert_eq!(tokenize(&input), &expected);
    }
}
