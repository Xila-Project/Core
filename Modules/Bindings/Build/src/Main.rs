#![allow(non_snake_case)]
use std::{env, path::Path, process::Command};

mod Bindings_generator;

use syn::visit::Visit;
use Bindings_generator::{Functions, Native, WASM};

fn main() -> Result<(), ()> {
    let Input = lvgl_rust_sys::_bindgen_raw_src();
    let Parsed_input = syn::parse_file(Input).expect("Error parsing input file");

    let mut LVGL_functions = Functions::LVGL_functions_type::default();
    LVGL_functions.visit_file(&Parsed_input);

    let Out_directory = env::var("OUT_DIR").unwrap();
    let Out_directory = Path::new(Out_directory.as_str());

    println!("cargo:warning=Output directory : {:?}", Out_directory);

    Native::Generate(&Out_directory, &LVGL_functions).expect("Error generating native bindings");
    WASM::Generate(&Out_directory, &LVGL_functions).expect("Error generating WASM bindings");

    println!("cargo:rerun-if-changed=Tests/WASM_test/src/main.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/src/Graphics.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/Cargo.toml");

    // TODO : Add a check for test mode

    if env::var("RUSTUP_TOOLCHAIN").unwrap().contains("esp") {
        println!("cargo:warning=Build of WASM tests are disabled for esp toolchain.");
        return Ok(());
    }

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir("Tests/WASM_test")
        .output()
        .unwrap();

    if !output.status.success() {
        println! {"cargo:warning=stderr: {}", String::from_utf8_lossy(&output.stderr)};
        println! {"cargo:warning=status: {}", output.status};
        return Err(());
    }

    Ok(())
}
