use std::{
    env, fs,
    path::{Path, PathBuf},
};

use bindings_utilities::format::format_rust;
use cbindgen::{EnumConfig, ExportConfig, FunctionConfig, RenameRule};
use quote::quote;
use syn::visit::Visit;
use target::Architecture;

mod generator;

fn is_c_bindings_enabled() -> bool {
    env::var_os("CARGO_FEATURE_C_BINDINGS").is_some()
}

fn generate_c_functions_module_body(path: impl AsRef<Path>) -> Result<(), String> {
    let token_stream = quote! {
        include!(concat!(env!("OUT_DIR"), "/c_functions.generated.rs"));
    };

    fs::write(&path, token_stream.to_string())
        .map_err(|e| format!("Error writing to file: {}", e))?;

    format_rust(path)?;

    Ok(())
}

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

    let out_directory = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crate_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let enumerations_generated_path = out_directory.join("enumeration.generated.rs");
    let functions_generated_path = out_directory.join("functions.generated.rs");
    let c_functions_generated_path = out_directory.join("c_functions.generated.rs");
    let c_functions_module_path = crate_directory.join("src").join("c_functions.rs");
    let c_header_path = out_directory.join("xila_graphics.h");

    generator::generate_enumeration(&enumerations_generated_path, &context).unwrap();

    generator::generate_functions(&functions_generated_path, &context).unwrap();

    if is_c_bindings_enabled() {
        // Overwrite c_functions.rs file with generated C ABI functions
        // This is workaround for cbindgen macro expansion limitations
        generator::generate_c_abi_functions(&c_functions_module_path, &context).unwrap();

        generator::generate_c_abi_functions(&c_functions_generated_path, &context).unwrap();

        let configuration: cbindgen::Config = cbindgen::Config {
            language: cbindgen::Language::C,
            include_guard: Some("__XILA_GRAPHICS_GENERATED_H_INCLUDED".to_string()),
            sys_includes: vec![
                "stdarg.h".to_string(),
                "stdbool.h".to_string(),
                "stdint.h".to_string(),
            ],
            export: ExportConfig {
                prefix: Some("XilaGraphics".to_string()),
                ..Default::default()
            },
            function: FunctionConfig {
                ..Default::default()
            },
            no_includes: true,
            enumeration: EnumConfig {
                rename_variants: RenameRule::QualifiedScreamingSnakeCase,
                ..Default::default()
            },
            ..Default::default()
        };

        cbindgen::Builder::new()
            .with_crate(crate_directory)
            .with_config(configuration)
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(&c_header_path);

        // Restore c_functions.rs file
        generate_c_functions_module_body(&c_functions_module_path).unwrap();
    }
}
