use std::{env, path::PathBuf};

pub fn main() {
    // Tell Cargo to rerun this build script if any Rust source files change
    let definitions_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../definitions");

    let declaration_file_path =
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("declarations.generated.rs");

    println!("cargo:rerun-if-changed={}", definitions_path.display());

    let header_file = format!("./xila.generated.h");

    let enumeration_configuration = cbindgen::EnumConfig {
        prefix_with_name: true,
        ..Default::default()
    };

    let configuration: cbindgen::Config = cbindgen::Config {
        language: cbindgen::Language::C,
        include_guard: Some("__XILA_GENERATED_H_INCLUDED".to_string()),
        sys_includes: vec![
            "stdarg.h".to_string(),
            "stdbool.h".to_string(),
            "stdint.h".to_string(),
        ],
        no_includes: true,
        enumeration: enumeration_configuration,
        ..Default::default()
    };

    cbindgen::Builder::new()
        .with_crate(definitions_path)
        .with_config(configuration)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&header_file);

    // Generate the bindings
    let mut bindings = bindgen::Builder::default()
        .header(header_file)
        .use_core()
        .generate_comments(true); // preserve comments

    if let Ok(target) = env::var("TARGET") {
        if target == "wasm32-unknown-unknown" {
            bindings = bindings.clang_arg("-fvisibility=default");
        }
    }

    bindings
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&declaration_file_path)
        .expect("Couldn't write bindings!");
}
