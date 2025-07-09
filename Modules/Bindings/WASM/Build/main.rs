#![allow(non_camel_case_types)]

use std::{env, path::Path};

use syn::visit::Visit;
use target::Architecture_type;

mod generator;

fn main() {
    // Build only for WASM32 architecture.
    if Architecture_type::get() != Architecture_type::WASM32 {
        return;
    }

    let input = lvgl_rust_sys::_bindgen_raw_src();
    let parsed_input = syn::parse_file(input).expect("Error parsing input file");

    let mut context = bindings_utilities::context::LVGL_context::default();
    context.set_function_filtering(Some(
        bindings_utilities::context::LVGL_context::filter_function,
    ));
    context.visit_file(&parsed_input);
    context.set_function_filtering(None);
    context.visit_file(&syn::parse2(bindings_utilities::additional::Get()).unwrap());

    let out_directory = env::var("OUT_DIR").unwrap();
    let out_directory = Path::new(out_directory.as_str());

    generator::generate(out_directory, &context).expect("Error generating WASM bindings");

    cc::Build::new()
        .file(out_directory.join("Xila_graphics.c"))
        .include(out_directory)
        .warnings(true)
        .compile("Xila_graphics");

    println!("cargo:rustc-link-search=native={}", out_directory.display());
    println!("cargo:rustc-link-lib=static=Xila_graphics");

    bindgen::builder()
        .header(out_directory.join("Xila_graphics.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core()
        .ctypes_prefix("::core::ffi")
        .clang_arg("-fvisibility=default")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_directory.join("Bindings.rs"))
        .expect("Unable to write bindings");
}
