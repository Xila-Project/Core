#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LeftParenthesis,
    RightParenthesis,
    Function(FunctionToken),
    Constant(ConstantToken),
    Identifier(&'a str),
    Eof,
}

impl<'a> Token<'a> {
    pub fn get_discriminant(&self) -> std::mem::Discriminant<Token<'a>> {
        std::mem::discriminant(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantToken {
    Pi,
    E,
}

impl TryFrom<&str> for ConstantToken {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pi" | "PI" => Ok(ConstantToken::Pi),
            "e" | "E" => Ok(ConstantToken::E),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionToken {
    SquareRoot,
    Sine,
    Cosine,
    Tangent,
    Logarithm,
    NaturalLogarithm,
    Exponential,
    AbsoluteValue,
    HyperbolicSine,
    HyperbolicCosine,
    HyperbolicTangent,
    Sind,
    Cosd,
    Tand,
    Sqr,
    Cube,
    Power10,
    Factorial,
    Inverse,
}

impl TryFrom<&str> for FunctionToken {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sqrt" => Ok(FunctionToken::SquareRoot),
            "sin" => Ok(FunctionToken::Sine),
            "cos" => Ok(FunctionToken::Cosine),
            "tan" => Ok(FunctionToken::Tangent),
            "log" => Ok(FunctionToken::Logarithm),
            "ln" => Ok(FunctionToken::NaturalLogarithm),
            "exp" => Ok(FunctionToken::Exponential),
            "abs" => Ok(FunctionToken::AbsoluteValue),
            "sinh" => Ok(FunctionToken::HyperbolicSine),
            "cosh" => Ok(FunctionToken::HyperbolicCosine),
            "tanh" => Ok(FunctionToken::HyperbolicTangent),
            "sind" => Ok(FunctionToken::Sind),
            "cosd" => Ok(FunctionToken::Cosd),
            "tand" => Ok(FunctionToken::Tand),
            "sqr" => Ok(FunctionToken::Sqr),
            "cube" => Ok(FunctionToken::Cube),
            "pow10" => Ok(FunctionToken::Power10),
            "fact" => Ok(FunctionToken::Factorial),
            "inv" => Ok(FunctionToken::Inverse),
            _ => Err(()),
        }
    }
}
