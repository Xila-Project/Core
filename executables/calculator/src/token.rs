#[derive(Debug, Clone, PartialEq)]
pub enum Token {
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
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();

        Self {
            input: chars,
            position: 0,
            current_char: current_char,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let mut number_str = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '.' {
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        number_str.parse().unwrap_or(0.0)
    }

    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphabetic() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        identifier
    }

    pub fn next_token(&mut self) -> Token {
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.skip_whitespace();
                    continue;
                }
                '+' => {
                    self.advance();
                    return Token::Plus;
                }
                '-' => {
                    self.advance();
                    return Token::Minus;
                }
                '*' | 'x' | 'X' => {
                    self.advance();
                    return Token::Multiply;
                }
                '/' => {
                    self.advance();
                    return Token::Divide;
                }
                '%' => {
                    self.advance();
                    return Token::Identifier("percent".to_string()); // Handle as special operator
                }
                '^' => {
                    self.advance();
                    return Token::Power;
                }
                '(' => {
                    self.advance();
                    return Token::LeftParen;
                }
                ')' => {
                    self.advance();
                    return Token::RightParen;
                }
                '0'..='9' => {
                    return Token::Number(self.read_number());
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let identifier = self.read_identifier();

                    // Check if it's a known function
                    match identifier.as_str() {
                        "sqrt" | "sin" | "cos" | "tan" | "log" | "ln" | "exp" | "abs" | "sinh"
                        | "cosh" | "tanh" | "sind" | "cosd" | "tand" | "sqr" | "cube" | "pow10"
                        | "fact" | "inv" => {
                            return Token::Function(identifier);
                        }
                        "mod" => {
                            return Token::Identifier(identifier); // Handle as identifier for binary operation
                        }
                        _ => {
                            return Token::Identifier(identifier);
                        }
                    }
                }
                _ => {
                    self.advance();
                    continue;
                }
            }
        }

        Token::EOF
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token == Token::EOF;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        tokens
    }
}
