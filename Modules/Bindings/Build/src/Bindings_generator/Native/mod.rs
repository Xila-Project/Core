#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{env, fs::File, io::Write, path::PathBuf, process::Command};

use super::Functions::LVGL_functions_type;
use quote::quote;

mod Call;
mod Enumeration;

pub fn Generate(LVGL_functions: &LVGL_functions_type) {
    // Open the output file
    let Output_file_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("Bindings.rs");

    let mut Output_file = File::create(&Output_file_path).expect("Error creating bindings file");

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

    // Output C header file for the bindings
    let Output_file_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("Bindings.h");

    let mut Output_file = File::create(&Output_file_path).expect("Error creating bindings file");
}
