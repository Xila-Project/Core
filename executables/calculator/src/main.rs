#![cfg(target_arch = "wasm32")]
mod evaluator;
mod install;
mod interface;
mod parser;
mod token;

fn main() {
    let mut calculator_gui = interface::Interface::new().unwrap();
    unsafe {
        calculator_gui.run();
    }
}
