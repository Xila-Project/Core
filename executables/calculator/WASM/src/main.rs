



pub mod evaluator;
pub mod interface;
pub mod parser;
pub mod token;

use evaluator::Evaluator_type;
use interface::Interface_type;
use parser::Parser_type;

pub struct Calculator;

impl Calculator {
    pub fn Evaluate_expression(input: &str) -> Result<f64, String> {
        // Parse the expression
        let mut Parser = Parser_type::new(input)?;
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
