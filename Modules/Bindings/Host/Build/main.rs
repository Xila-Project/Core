use std::{env, path::Path};

use syn::visit::Visit;
mod generator;

fn main() -> Result<(), ()> {
    let input = lvgl_rust_sys::_bindgen_raw_src();
    let parsed_input = syn::parse_file(input).expect("Error parsing input file");

    let mut context = bindings_utilities::context::LvglContext::default();
    context.set_function_filtering(Some(
        bindings_utilities::context::LvglContext::filter_function,
    ));
    context.visit_file(&parsed_input);
    context.set_function_filtering(None);
    context.visit_file(&syn::parse2(bindings_utilities::additional::get()).unwrap());

    let out_directory = env::var("OUT_DIR").unwrap();
    let out_directory = Path::new(out_directory.as_str());

    generator::generate(out_directory, &context).expect("Error generating native bindings");

    println!("cargo:rerun-if-changed=Tests/WASM_test/src/main.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/Cargo.toml");

    // TODO : Add a check for test mode

    //    if env::var("RUSTUP_TOOLCHAIN").unwrap().contains("esp") {
    //        println!("cargo:warning=Build of WASM tests are disabled for esp toolchain.");
    //    } else {
    //        let output = Command::new("cargo")
    //            .args(["build", "--release"])
    //            .current_dir("Tests/WASM_test")
    //            .output()
    //            .unwrap();
    //
    //        if !output.status.success() {
    //            println! {"cargo:warning=stderr: {}", String::from_utf8_lossy(&output.stderr)};
    //            println! {"cargo:warning=status: {}", output.status};
    //            return Err(());
    //        }
    //    }

    Ok(())
}
