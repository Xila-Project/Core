use crate::Parser::{Binary_operator_type, Expression_type, Unary_operator_type};

pub struct Evaluator_type;

impl Evaluator_type {
    pub fn Evaluate(Expression: &Expression_type) -> Result<f64, String> {
        match Expression {
            Expression_type::Number(Value) => Ok(*Value),
            Expression_type::BinaryOp { left, op, right } => {
                let Left_val = Self::Evaluate(left)?;
                let Right_val = Self::Evaluate(right)?;

                match op {
                    Binary_operator_type::Add => Ok(Left_val + Right_val),
                    Binary_operator_type::Subtract => Ok(Left_val - Right_val),
                    Binary_operator_type::Multiply => Ok(Left_val * Right_val),
                    Binary_operator_type::Divide => {
                        if Right_val == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Left_val / Right_val)
                        }
                    }
                    Binary_operator_type::Power => Ok(Left_val.powf(Right_val)),
                    Binary_operator_type::Modulo => {
                        if Right_val == 0.0 {
                            Err("Modulo by zero".to_string())
                        } else {
                            Ok(Left_val % Right_val)
                        }
                    }
                    Binary_operator_type::Percent => {
                        // Percent operation: left% = left/100
                        Ok(Left_val / 100.0)
                    }
                }
            }
            Expression_type::UnaryOp { op, expr } => {
                let Val = Self::Evaluate(expr)?;

                match op {
                    Unary_operator_type::Plus => Ok(Val),
                    Unary_operator_type::Minus => Ok(-Val),
                }
            }
            Expression_type::FunctionCall { name, arg } => {
                let Arg_val = Self::Evaluate(arg)?;

                match name.as_str() {
                    "sqrt" => {
                        if Arg_val < 0.0 {
                            Err("Square root of negative number".to_string())
                        } else {
                            Ok(Arg_val.sqrt())
                        }
                    }
                    "sin" => Ok(Arg_val.sin()),
                    "cos" => Ok(Arg_val.cos()),
                    "tan" => Ok(Arg_val.tan()),
                    "sind" => Ok((Arg_val * std::f64::consts::PI / 180.0).sin()), // degrees
                    "cosd" => Ok((Arg_val * std::f64::consts::PI / 180.0).cos()), // degrees
                    "tand" => Ok((Arg_val * std::f64::consts::PI / 180.0).tan()), // degrees
                    "sinh" => Ok(Arg_val.sinh()),
                    "cosh" => Ok(Arg_val.cosh()),
                    "tanh" => Ok(Arg_val.tanh()),
                    "log" => {
                        if Arg_val <= 0.0 {
                            Err("Logarithm of non-positive number".to_string())
                        } else {
                            Ok(Arg_val.log10())
                        }
                    }
                    "ln" => {
                        if Arg_val <= 0.0 {
                            Err("Natural logarithm of non-positive number".to_string())
                        } else {
                            Ok(Arg_val.ln())
                        }
                    }
                    "exp" => Ok(Arg_val.exp()),
                    "abs" => Ok(Arg_val.abs()),
                    "sqr" => Ok(Arg_val * Arg_val),
                    "cube" => Ok(Arg_val * Arg_val * Arg_val),
                    "pow10" => Ok(10.0_f64.powf(Arg_val)),
                    "fact" => {
                        if Arg_val < 0.0 || Arg_val.fract() != 0.0 {
                            Err("Factorial requires non-negative integer".to_string())
                        } else if Arg_val > 170.0 {
                            Err("Factorial too large".to_string())
                        } else {
                            let n = Arg_val as u64;
                            let mut result = 1.0;
                            for i in 1..=n {
                                result *= i as f64;
                            }
                            Ok(result)
                        }
                    }
                    "inv" => {
                        if Arg_val == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(1.0 / Arg_val)
                        }
                    }
                    _ => Err(format!("Unknown function: {}", name)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser::Parser_type;

    fn Evaluate_expression(Input: &str) -> Result<f64, String> {
        let mut Parser = Parser_type::new(Input)?;
        let Expression = Parser.Parse()?;
        Evaluator_type::Evaluate(&Expression)
    }

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(Evaluate_expression("2 + 3").unwrap(), 5.0);
        assert_eq!(Evaluate_expression("10 - 4").unwrap(), 6.0);
        assert_eq!(Evaluate_expression("6 * 7").unwrap(), 42.0);
        assert_eq!(Evaluate_expression("15 / 3").unwrap(), 5.0);
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(Evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(Evaluate_expression("2 + (3 * 4)").unwrap(), 14.0);
    }

    #[test]
    fn test_power() {
        assert_eq!(Evaluate_expression("2 ^ 3").unwrap(), 8.0);
        assert_eq!(Evaluate_expression("10 ^ 2").unwrap(), 100.0);
    }

    #[test]
    fn test_functions() {
        assert_eq!(Evaluate_expression("sqrt(16)").unwrap(), 4.0);
        assert_eq!(Evaluate_expression("abs(-5)").unwrap(), 5.0);
    }

    #[test]
    fn test_complex_expression() {
        // Test the target expression: ((5 / 8) + sqrt(18) x 10^5 ) / 23
        let Result = Evaluate_expression("((5 / 8) + sqrt(18) * 10^5) / 23").unwrap();

        // Calculate expected result step by step
        let Part1 = 5.0 / 8.0; // 0.625
        let Part2 = 18.0_f64.sqrt() * (10.0_f64.powf(5.0)); // sqrt(18) * 100000
        let Numerator = Part1 + Part2;
        let Expected = Numerator / 23.0;

        assert!((Result - Expected).abs() < 1e-10);
    }

    #[test]
    fn test_new_functions() {
        // Test hyperbolic functions
        assert!((Evaluate_expression("sinh(0)").unwrap() - 0.0).abs() < 1e-10);
        assert!((Evaluate_expression("cosh(0)").unwrap() - 1.0).abs() < 1e-10);
        assert!((Evaluate_expression("tanh(0)").unwrap() - 0.0).abs() < 1e-10);

        // Test square and cube
        assert_eq!(Evaluate_expression("sqr(5)").unwrap(), 25.0);
        assert_eq!(Evaluate_expression("cube(3)").unwrap(), 27.0);

        // Test factorial
        assert_eq!(Evaluate_expression("fact(5)").unwrap(), 120.0);
        assert_eq!(Evaluate_expression("fact(0)").unwrap(), 1.0);

        // Test power of 10
        assert_eq!(Evaluate_expression("pow10(2)").unwrap(), 100.0);
        assert_eq!(Evaluate_expression("pow10(0)").unwrap(), 1.0);

        // Test inverse
        assert_eq!(Evaluate_expression("inv(2)").unwrap(), 0.5);
        assert_eq!(Evaluate_expression("inv(4)").unwrap(), 0.25);
        assert!((Evaluate_expression("inv(3)").unwrap() - (1.0 / 3.0)).abs() < 1e-10);

        // Test absolute value
        assert_eq!(Evaluate_expression("abs(-5)").unwrap(), 5.0);
        assert_eq!(Evaluate_expression("abs(3)").unwrap(), 3.0);
        assert_eq!(Evaluate_expression("abs(0)").unwrap(), 0.0);

        // Test modulo
        assert_eq!(Evaluate_expression("10 mod 3").unwrap(), 1.0);
        assert_eq!(Evaluate_expression("7 mod 2").unwrap(), 1.0);
        assert_eq!(Evaluate_expression("8 mod 4").unwrap(), 0.0);

        // Test percent
        assert_eq!(Evaluate_expression("50 percent 1").unwrap(), 0.5);
        assert_eq!(Evaluate_expression("25 percent 1").unwrap(), 0.25);

        // Test degree trigonometric functions
        assert!((Evaluate_expression("sind(90)").unwrap() - 1.0).abs() < 1e-10);
        assert!((Evaluate_expression("cosd(0)").unwrap() - 1.0).abs() < 1e-10);
        assert!((Evaluate_expression("tand(45)").unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_cases() {
        // Test factorial with negative number
        assert!(Evaluate_expression("fact(-1)").is_err());

        // Test factorial with decimal
        assert!(Evaluate_expression("fact(3.5)").is_err());

        // Test factorial with large number
        assert!(Evaluate_expression("fact(200)").is_err());

        // Test inverse with zero
        assert!(Evaluate_expression("inv(0)").is_err());

        // Test modulo with zero
        assert!(Evaluate_expression("5 mod 0").is_err());
    }
}
