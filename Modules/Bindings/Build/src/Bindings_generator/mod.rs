#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{env, fs::File, io::Write, path::PathBuf, process::Command};

use quote::quote;
use syn::visit::Visit;
use Functions::LVGL_functions_type;

mod Call;
mod Enumeration;
mod Functions;
mod Type_tree;

pub fn Generate() {
    // Parse the input file
    let String = lvgl_rust_sys::_bindgen_raw_src();
    let File = syn::parse_str(String).expect("Error parsing lvgl bindings");

    // Open the output file
    let Output_file_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("Bindings.rs");

    let mut Output_file = File::create(&Output_file_path).expect("Error creating bindings file");

    let mut LVGL_functions = LVGL_functions_type::default();
    LVGL_functions.visit_file(&File);

    let Enumerations = Enumeration::Generate_code(LVGL_functions.Get_signatures());

    let Functions = Call::Generate_code(
        LVGL_functions.Get_type_tree(),
        LVGL_functions.Get_signatures(),
    );

    Output_file
        .write_all(
            quote! {
                mod Generated_bindings {
                    #Enumerations

                    #Functions
                }
            }
            .to_string()
            .as_bytes(),
        )
        .expect("Error writing to bindings file");

    Command::new("rustfmt")
        .arg(Output_file_path.to_str().unwrap())
        .status()
        .expect("Error running rustfmt");
}
