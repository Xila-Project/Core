#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub fn main() {
    let Out_dir = "./include";
    let Out_file = format!("{Out_dir}/Xila.h");

    let Enumeration_configuration = cbindgen::EnumConfig {
        prefix_with_name: true,
        ..Default::default()
    };

    let Configuration: cbindgen::Config = cbindgen::Config {
        language: cbindgen::Language::C,
        include_guard: Some("XILA_H_INCLUDED".to_string()),
        enumeration: Enumeration_configuration,
        ..Default::default()
    };

    cbindgen::Builder::new()
        .with_crate(".")
        .with_config(Configuration)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&Out_file);
}
