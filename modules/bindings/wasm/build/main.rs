use std::{env, path::Path};

use syn::visit::Visit;
use target::Architecture;

mod generator;

fn main() {
    // Build only for WASM32 architecture.
    if Architecture::get() != Architecture::WASM32 {
        return;
    }

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

    generator::generate(out_directory, &context).expect("Error generating WASM bindings");

    cc::Build::new()
        .file(out_directory.join("xila_graphics.c"))
        .include(out_directory)
        .warnings(true)
        .compile("xila_graphics");

    println!("cargo:rustc-link-search=native={}", out_directory.display());
    println!("cargo:rustc-link-lib=static=xila_graphics");

    bindgen::builder()
        .header(out_directory.join("xila_graphics.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core()
        .ctypes_prefix("::core::ffi")
        .clang_arg("-fvisibility=default")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_directory.join("bindings.rs"))
        .expect("Unable to write bindings");
}
