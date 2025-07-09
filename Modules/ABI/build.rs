#![allow(non_camel_case_types)]

pub fn main() {
    let out_dir = "./include";
    let out_file = format!("{out_dir}/Xila.h");

    let enumeration_configuration = cbindgen::EnumConfig {
        prefix_with_name: true,
        ..Default::default()
    };

    let configuration: cbindgen::Config = cbindgen::Config {
        language: cbindgen::Language::C,
        include_guard: Some("XILA_H_INCLUDED".to_string()),
        enumeration: enumeration_configuration,
        ..Default::default()
    };

    cbindgen::Builder::new()
        .with_crate(".")
        .with_config(configuration)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&out_file);
}
