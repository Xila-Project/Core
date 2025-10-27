use crate::token::{FunctionToken, Token};

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let current_char = input.chars().next();

        Lexer {
            input,
            position: 0,
            current_char,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
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
        let start_position = self.position;

        let end_position = self.input[self.position..]
            .chars()
            .position(|ch| !(ch.is_ascii_digit() || ch == '.'))
            .map_or(self.input.len(), |pos| self.position + pos);

        let number_str = &self.input[start_position..end_position];
        self.position = end_position;
        self.current_char = self.input.chars().nth(self.position);
        number_str.parse().unwrap_or(0.0)
    }

    fn read_identifier(&mut self) -> &'a str {
        let start_position = self.position;

        let end_position = self.input[self.position..]
            .chars()
            .enumerate()
            .position(|(i, ch)| {
                !(ch.is_ascii_alphabetic() || ch == '_' || (ch.is_ascii_digit() && i > 0))
            })
            .map_or(self.input.len(), |pos| self.position + pos);

        let identifier = &self.input[start_position..end_position];
        self.position = end_position;
        self.current_char = self.input.chars().nth(self.position);
        identifier
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.skip_whitespace();
                    continue;
                }
                '+' => {
                    self.advance();
                    return Some(Token::Plus);
                }
                '-' => {
                    self.advance();
                    return Some(Token::Minus);
                }
                '*' => {
                    self.advance();
                    return Some(Token::Multiply);
                }
                '/' => {
                    self.advance();
                    return Some(Token::Divide);
                }
                '%' => {
                    self.advance();
                    return Some(Token::Identifier("percent")); // Handle as special operator
                }
                '^' => {
                    self.advance();
                    return Some(Token::Power);
                }
                '(' => {
                    self.advance();
                    return Some(Token::LeftParenthesis);
                }
                ')' => {
                    self.advance();
                    return Some(Token::RightParenthesis);
                }
                '0'..='9' | '.' => {
                    return Some(Token::Number(self.read_number()));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let identifier = self.read_identifier();

                    // Check if it's a known function
                    if let Ok(function) = FunctionToken::try_from(identifier) {
                        return Some(Token::Function(function));
                    } else if let Ok(constant) = crate::token::ConstantToken::try_from(identifier) {
                        return Some(Token::Constant(constant));
                    } else {
                        return Some(Token::Identifier(identifier));
                    }
                }
                _ => {
                    self.advance();
                    continue;
                }
            }
        }

        None
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operators() {
        let lexer = Lexer::new("+ - * / ^ ( )");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Multiply,
                Token::Divide,
                Token::Power,
                Token::LeftParenthesis,
                Token::RightParenthesis,
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let lexer = Lexer::new("42 3.14 0.5 100");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Number(42.0),
                Token::Number(3.14),
                Token::Number(0.5),
                Token::Number(100.0),
            ]
        );
    }

    #[test]
    fn test_functions() {
        let lexer = Lexer::new("sqrt sin cos tan log ln exp abs");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Function(FunctionToken::SquareRoot),
                Token::Function(FunctionToken::Sine),
                Token::Function(FunctionToken::Cosine),
                Token::Function(FunctionToken::Tangent),
                Token::Function(FunctionToken::Logarithm),
                Token::Function(FunctionToken::NaturalLogarithm),
                Token::Function(FunctionToken::Exponential),
                Token::Function(FunctionToken::AbsoluteValue),
            ]
        );
    }

    #[test]
    fn test_hyperbolic_functions() {
        let lexer = Lexer::new("sinh cosh tanh");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Function(FunctionToken::HyperbolicSine),
                Token::Function(FunctionToken::HyperbolicCosine),
                Token::Function(FunctionToken::HyperbolicTangent),
            ]
        );
    }

    #[test]
    fn test_degree_functions() {
        let lexer = Lexer::new("sind cosd tand");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Function(FunctionToken::Sind),
                Token::Function(FunctionToken::Cosd),
                Token::Function(FunctionToken::Tand),
            ]
        );
    }

    #[test]
    fn test_special_functions() {
        let lexer = Lexer::new("sqr cube pow10 fact inv");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Function(FunctionToken::Sqr),
                Token::Function(FunctionToken::Cube),
                Token::Function(FunctionToken::Power10),
                Token::Function(FunctionToken::Factorial),
                Token::Function(FunctionToken::Inverse),
            ]
        );
    }

    #[test]
    fn test_expression() {
        let lexer = Lexer::new("2 + 3 * 4");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Number(2.0),
                Token::Plus,
                Token::Number(3.0),
                Token::Multiply,
                Token::Number(4.0),
            ]
        );
    }

    #[test]
    fn test_function_call() {
        let lexer = Lexer::new("sqrt(16)");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Function(FunctionToken::SquareRoot),
                Token::LeftParenthesis,
                Token::Number(16.0),
                Token::RightParenthesis,
            ]
        );
    }

    #[test]
    fn test_modulo_and_percent() {
        let lexer = Lexer::new("10 mod 3 + 50%");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Number(10.0),
                Token::Identifier("mod"),
                Token::Number(3.0),
                Token::Plus,
                Token::Number(50.0),
                Token::Identifier("percent"),
            ]
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let lexer = Lexer::new("  2  +  3  ");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![Token::Number(2.0), Token::Plus, Token::Number(3.0),]
        );
    }

    #[test]
    fn test_decimal_numbers() {
        let lexer = Lexer::new("0.1 .5 123.456");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Number(0.1),
                Token::Number(0.5),
                Token::Number(123.456),
            ]
        );
    }

    #[test]
    fn test_unknown_identifiers() {
        let lexer = Lexer::new("x + y");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![Token::Identifier("x"), Token::Plus, Token::Identifier("y"),]
        );
    }

    #[test]
    fn test_nested_parentheses() {
        let lexer = Lexer::new("((2 + 3) * 4)");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParenthesis,
                Token::LeftParenthesis,
                Token::Number(2.0),
                Token::Plus,
                Token::Number(3.0),
                Token::RightParenthesis,
                Token::Multiply,
                Token::Number(4.0),
                Token::RightParenthesis,
            ]
        );
    }

    #[test]
    fn test_power_operator() {
        let lexer = Lexer::new("2^3^4");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Number(2.0),
                Token::Power,
                Token::Number(3.0),
                Token::Power,
                Token::Number(4.0),
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let lexer = Lexer::new("");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_complex_expression() {
        let lexer = Lexer::new("sin(45) + sqrt(16) * 2 - log(100)");
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Function(FunctionToken::Sine),
                Token::LeftParenthesis,
                Token::Number(45.0),
                Token::RightParenthesis,
                Token::Plus,
                Token::Function(FunctionToken::SquareRoot),
                Token::LeftParenthesis,
                Token::Number(16.0),
                Token::RightParenthesis,
                Token::Multiply,
                Token::Number(2.0),
                Token::Minus,
                Token::Function(FunctionToken::Logarithm),
                Token::LeftParenthesis,
                Token::Number(100.0),
                Token::RightParenthesis,
            ]
        );
    }
}
