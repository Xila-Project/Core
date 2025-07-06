#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod Evaluator;
pub mod Interface;
pub mod Parser;
pub mod Token;

use Evaluator::Evaluator_type;
use Interface::Interface_type;
use Parser::Parser_type;

pub struct Calculator;

impl Calculator {
    pub fn Evaluate_expression(input: &str) -> Result<f64, String> {
        // Parse the expression
        let mut Parser = Parser_type::New(input)?;
        let Expr = Parser.Parse()?;

        // Evaluate the parsed expression
        Evaluator_type::Evaluate(&Expr)
    }

    pub fn Format_result(result: f64) -> String {
        if result.fract() == 0.0 && result.abs() < 1e15 {
            format!("{:.0}", result)
        } else {
            format!("{:.10}", result)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    }
}

fn main() {
    let mut calculator_gui = Interface_type::new();
    unsafe {
        calculator_gui.run();
    }
}
