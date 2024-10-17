#![allow(non_snake_case)]
use std::{env, process::Command};

use syn::visit::Visit;
use Bindings_generator::{LVGL_functions_type, Native, WASM};

mod Bindings_generator;

fn main() -> Result<(), ()> {
    println!("cargo:rerun-if-changed=Tests/WASM_test/src/main.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/src/File_system.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/src/Task.rs");
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

    // Parse the input file
    let String = lvgl_rust_sys::_bindgen_raw_src();
    let File = syn::parse_str(String).expect("Error parsing lvgl bindings");

    let mut LVGL_functions = LVGL_functions_type::default();
    LVGL_functions.visit_file(&File);

    Native::Generate(&LVGL_functions);
    WASM::Generate_header(&LVGL_functions);
    WASM::Generate_source(&LVGL_functions);

    Ok(())
}
