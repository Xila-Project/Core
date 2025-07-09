use crate::Token::{Lexer_type, Token_type};

#[derive(Debug, Clone)]
pub enum Expression_type {
    Number(f64),
    BinaryOp {
        left: Box<Expression_type>,
        op: Binary_operator_type,
        right: Box<Expression_type>,
    },
    UnaryOp {
        op: Unary_operator_type,
        expr: Box<Expression_type>,
    },
    FunctionCall {
        name: String,
        arg: Box<Expression_type>,
    },
}

#[derive(Debug, Clone)]
pub enum Binary_operator_type {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    Percent,
}

#[derive(Debug, Clone)]
pub enum Unary_operator_type {
    Plus,
    Minus,
}

#[derive(Debug)]
pub struct Parser_type {
    tokens: Vec<Token_type>,
    position: usize,
}

impl Parser_type {
    pub fn New(Input: &str) -> Result<Self, String> {
        let mut Lexer = Lexer_type::New(Input);
        let Tokens = Lexer.tokenize();

        Ok(Self {
            tokens: Tokens,
            position: 0,
        })
    }

    fn current_token(&self) -> &Token_type {
        self.tokens.get(self.position).unwrap_or(&Token_type::EOF)
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect_token(&mut self, Expected: Token_type) -> Result<(), String> {
        if std::mem::discriminant(self.current_token()) == std::mem::discriminant(&Expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                Expected,
                self.current_token()
            ))
        }
    }

    pub fn Parse(&mut self) -> Result<Expression_type, String> {
        self.parse_expression()
    }

    // Expression = Term (('+' | '-') Term)*
    fn parse_expression(&mut self) -> Result<Expression_type, String> {
        let mut Left = self.parse_term()?;

        while matches!(self.current_token(), Token_type::Plus | Token_type::Minus) {
            let Op = match self.current_token() {
                Token_type::Plus => Binary_operator_type::Add,
                Token_type::Minus => Binary_operator_type::Subtract,
                _ => unreachable!(),
            };
            self.advance();
            let Right = self.parse_term()?;
            Left = Expression_type::BinaryOp {
                left: Box::new(Left),
                op: Op,
                right: Box::new(Right),
            };
        }

        Ok(Left)
    }

    // Term = Factor (('*' | '/' | 'mod' | '%') Factor)*
    fn parse_term(&mut self) -> Result<Expression_type, String> {
        let mut Left = self.parse_factor()?;

        loop {
            let Op = match self.current_token() {
                Token_type::Multiply => Binary_operator_type::Multiply,
                Token_type::Divide => Binary_operator_type::Divide,
                Token_type::Identifier(s) if s == "mod" => Binary_operator_type::Modulo,
                Token_type::Identifier(s) if s == "percent" => Binary_operator_type::Percent,
                _ => break,
            };

            self.advance();
            let Right = self.parse_factor()?;
            Left = Expression_type::BinaryOp {
                left: Box::new(Left),
                op: Op,
                right: Box::new(Right),
            };
        }

        Ok(Left)
    }

    // Factor = Power ('^' Power)*
    fn parse_factor(&mut self) -> Result<Expression_type, String> {
        let mut Left = self.parse_power()?;

        while matches!(self.current_token(), Token_type::Power) {
            self.advance();
            let Right = self.parse_power()?;
            Left = Expression_type::BinaryOp {
                left: Box::new(Left),
                op: Binary_operator_type::Power,
                right: Box::new(Right),
            };
        }

        Ok(Left)
    }

    // Power = ('+' | '-')? Primary
    fn parse_power(&mut self) -> Result<Expression_type, String> {
        match self.current_token() {
            Token_type::Plus => {
                self.advance();
                let Expression = self.parse_primary()?;
                Ok(Expression_type::UnaryOp {
                    op: Unary_operator_type::Plus,
                    expr: Box::new(Expression),
                })
            }
            Token_type::Minus => {
                self.advance();
                let Expression = self.parse_primary()?;
                Ok(Expression_type::UnaryOp {
                    op: Unary_operator_type::Minus,
                    expr: Box::new(Expression),
                })
            }
            _ => self.parse_primary(),
        }
    }

    // Primary = Number | '(' Expression ')' | Function '(' Expression ')' | Identifier
    fn parse_primary(&mut self) -> Result<Expression_type, String> {
        match self.current_token().clone() {
            Token_type::Number(Value) => {
                self.advance();
                Ok(Expression_type::Number(Value))
            }
            Token_type::LeftParen => {
                self.advance();
                let Expression = self.parse_expression()?;
                self.expect_token(Token_type::RightParen)?;
                Ok(Expression)
            }
            Token_type::Function(Name) => {
                self.advance();
                self.expect_token(Token_type::LeftParen)?;
                let Arg = self.parse_expression()?;
                self.expect_token(Token_type::RightParen)?;
                Ok(Expression_type::FunctionCall {
                    name: Name,
                    arg: Box::new(Arg),
                })
            }
            Token_type::Identifier(Name) => {
                self.advance();
                // Handle constants like pi, e
                match Name.as_str() {
                    "pi" | "PI" => Ok(Expression_type::Number(std::f64::consts::PI)),
                    "e" | "E" => Ok(Expression_type::Number(std::f64::consts::E)),
                    _ => Err(format!("Unknown identifier: {}", Name)),
                }
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }
}
