#![allow(non_camel_case_types)]

use std::{env, path::Path};

use syn::visit::Visit;
use Target::Architecture_type;

mod Generator;

fn main() {
    // Build only for WASM32 architecture.
    if Architecture_type::get() != Architecture_type::WASM32 {
        return;
    }

    let Input = lvgl_rust_sys::_bindgen_raw_src();
    let parsed_input = syn::parse_file(Input).expect("Error parsing input file");

    let mut Context = Bindings_utilities::Context::LVGL_context::default();
    Context.set_function_filtering(Some(
        Bindings_utilities::Context::LVGL_context::Filter_function,
    ));
    Context.visit_file(&parsed_input);
    Context.set_function_filtering(None);
    Context.visit_file(&syn::parse2(Bindings_utilities::Additional::Get()).unwrap());

    let Out_directory = env::var("OUT_DIR").unwrap();
    let out_directory = Path::new(Out_directory.as_str());

    Generator::generate(out_directory, &Context).expect("Error generating WASM bindings");

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
