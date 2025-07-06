#[derive(Debug, Clone, PartialEq)]
pub enum Token_type {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LeftParen,
    RightParen,
    Function(String),
    Identifier(String),
    EOF,
}

#[derive(Debug, Clone)]
pub struct Lexer_type {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer_type {
    pub fn New(Input: &str) -> Self {
        let Chars: Vec<char> = Input.chars().collect();
        let Current_char = Chars.get(0).copied();

        Self {
            input: Chars,
            position: 0,
            current_char: Current_char,
        }
    }

    fn Advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn skip_whitespace(&mut self) {
        while let Some(Ch) = self.current_char {
            if Ch.is_whitespace() {
                self.Advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let mut Number_str = String::new();

        while let Some(Ch) = self.current_char {
            if Ch.is_ascii_digit() || Ch == '.' {
                Number_str.push(Ch);
                self.Advance();
            } else {
                break;
            }
        }

        Number_str.parse().unwrap_or(0.0)
    }

    fn read_identifier(&mut self) -> String {
        let mut Identifier = String::new();

        while let Some(Ch) = self.current_char {
            if Ch.is_ascii_alphabetic() || Ch == '_' {
                Identifier.push(Ch);
                self.Advance();
            } else {
                break;
            }
        }

        Identifier
    }

    pub fn next_token(&mut self) -> Token_type {
        while let Some(Ch) = self.current_char {
            match Ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.skip_whitespace();
                    continue;
                }
                '+' => {
                    self.Advance();
                    return Token_type::Plus;
                }
                '-' => {
                    self.Advance();
                    return Token_type::Minus;
                }
                '*' | 'x' | 'X' => {
                    self.Advance();
                    return Token_type::Multiply;
                }
                '/' => {
                    self.Advance();
                    return Token_type::Divide;
                }
                '%' => {
                    self.Advance();
                    return Token_type::Identifier("percent".to_string()); // Handle as special operator
                }
                '^' => {
                    self.Advance();
                    return Token_type::Power;
                }
                '(' => {
                    self.Advance();
                    return Token_type::LeftParen;
                }
                ')' => {
                    self.Advance();
                    return Token_type::RightParen;
                }
                '0'..='9' => {
                    return Token_type::Number(self.read_number());
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let Identifier = self.read_identifier();

                    // Check if it's a known function
                    match Identifier.as_str() {
                        "sqrt" | "sin" | "cos" | "tan" | "log" | "ln" | "exp" | "abs" | "sinh"
                        | "cosh" | "tanh" | "sind" | "cosd" | "tand" | "sqr" | "cube" | "pow10"
                        | "fact" | "inv" => {
                            return Token_type::Function(Identifier);
                        }
                        "mod" => {
                            return Token_type::Identifier(Identifier); // Handle as identifier for binary operation
                        }
                        _ => {
                            return Token_type::Identifier(Identifier);
                        }
                    }
                }
                _ => {
                    self.Advance();
                    continue;
                }
            }
        }

        Token_type::EOF
    }

    pub fn tokenize(&mut self) -> Vec<Token_type> {
        let mut Tokens = Vec::new();

        loop {
            let Token = self.next_token();
            let Is_eof = Token == Token_type::EOF;
            Tokens.push(Token);

            if Is_eof {
                break;
            }
        }

        Tokens
    }
}
