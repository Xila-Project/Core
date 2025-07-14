use crate::token::{Lexer, Token};

#[derive(Debug, Clone)]
pub enum Expression {
    Number(f64),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arg: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    Percent,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Plus,
    Minus,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        Ok(Self {
            tokens,
            position: 0,
        })
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(self.current_token()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                expected,
                self.current_token()
            ))
        }
    }

    pub fn parse(&mut self) -> Result<Expression, String> {
        self.parse_expression()
    }

    // Expression = Term (('+' | '-') Term)*
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_term()?;

        while matches!(self.current_token(), Token::Plus | Token::Minus) {
            let op = match self.current_token() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Term = Factor (('*' | '/' | 'mod' | '%') Factor)*
    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_factor()?;

        loop {
            let op = match self.current_token() {
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                Token::Identifier(s) if s == "mod" => BinaryOperator::Modulo,
                Token::Identifier(s) if s == "percent" => BinaryOperator::Percent,
                _ => break,
            };

            self.advance();
            let right = self.parse_factor()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Factor = Power ('^' Power)*
    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_power()?;

        while matches!(self.current_token(), Token::Power) {
            self.advance();
            let right = self.parse_power()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Power,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Power = ('+' | '-')? Primary
    fn parse_power(&mut self) -> Result<Expression, String> {
        match self.current_token() {
            Token::Plus => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Plus,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Minus,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    // Primary = Number | '(' Expression ')' | Function '(' Expression ')' | Identifier
    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.current_token().clone() {
            Token::Number(value) => {
                self.advance();
                Ok(Expression::Number(value))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            Token::Function(name) => {
                self.advance();
                self.expect_token(Token::LeftParen)?;
                let arg = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(Expression::FunctionCall {
                    name: name,
                    arg: Box::new(arg),
                })
            }
            Token::Identifier(name) => {
                self.advance();
                // Handle constants like pi, e
                match name.as_str() {
                    "pi" | "PI" => Ok(Expression::Number(std::f64::consts::PI)),
                    "e" | "E" => Ok(Expression::Number(std::f64::consts::E)),
                    _ => Err(format!("Unknown identifier: {}", name)),
                }
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }
}
