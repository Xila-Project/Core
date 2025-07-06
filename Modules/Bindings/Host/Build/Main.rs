#![allow(non_snake_case)]
use std::{env, path::Path, process::Command};

use syn::visit::Visit;
mod Generator;

fn main() -> Result<(), ()> {
    let Input = lvgl_rust_sys::_bindgen_raw_src();
    let Parsed_input = syn::parse_file(Input).expect("Error parsing input file");

    let mut Context = Bindings_utilities::Context::LVGL_context::default();
    Context.Set_function_filtering(Some(
        Bindings_utilities::Context::LVGL_context::Filter_function,
    ));
    Context.visit_file(&Parsed_input);
    Context.Set_function_filtering(None);
    Context.visit_file(&syn::parse2(Bindings_utilities::Additional::Get()).unwrap());

    let Out_directory = env::var("OUT_DIR").unwrap();
    let Out_directory = Path::new(Out_directory.as_str());

    Generator::Generate(Out_directory, &Context).expect("Error generating native bindings");

    println!("cargo:rerun-if-changed=Tests/WASM_test/src/main.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/src/Graphics.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/Cargo.toml");

    // TODO : Add a check for test mode

    if env::var("RUSTUP_TOOLCHAIN").unwrap().contains("esp") {
        println!("cargo:warning=Build of WASM tests are disabled for esp toolchain.");
    } else {
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
    }

    Ok(())
}
