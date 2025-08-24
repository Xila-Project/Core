pub fn main() {
    let out_dir = "./include";
    let out_file = format!("{out_dir}/__xila_abi_generated.h");

    let enumeration_configuration = cbindgen::EnumConfig {
        prefix_with_name: true,
        ..Default::default()
    };

    let configuration: cbindgen::Config = cbindgen::Config {
        language: cbindgen::Language::C,
        include_guard: Some("__XILA_GENERATED_H_INCLUDED".to_string()),
        enumeration: enumeration_configuration,
        sys_includes: vec![
            "stddef.h".to_string(),
            "stdint.h".to_string(),
            "stdbool.h".to_string(),
        ],
        no_includes: true,
        ..Default::default()
    };

    cbindgen::Builder::new()
        .with_crate(".")
        .with_config(configuration)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&out_file);
}
