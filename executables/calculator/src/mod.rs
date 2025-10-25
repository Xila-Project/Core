pub mod evaluator;
pub mod interface;
pub mod parser;
pub mod token;

pub use interface::Interface;

use crate::wasm::{evaluator::Evaluator, parser::Parser};

pub fn evaluate_expression(input: &str) -> Result<f64, String> {
    // Parse the expression
    let mut parser = Parser::new(input)?;
    let expression = parser.parse()?;

    // Evaluate the parsed expression
    Evaluator::evaluate(&expression)
}

pub fn format_result(result: f64) -> String {
    if result.fract() == 0.0 && result.abs() < 1e15 {
        format!("{:.0}", result)
    } else {
        format!("{:.10}", result)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
