#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{fs::File, io::Write, path::Path};

use super::{Enumeration, Format::Format_rust, Functions::LVGL_functions_type};
use quote::quote;

mod Call;

pub fn Generate(Output_path: &Path, LVGL_functions: &LVGL_functions_type) -> Result<(), String> {
    // Open the output file
    let Output_file_path = Output_path.join("Bindings.rs");
    let mut Output_file = File::create(&Output_file_path)
        .map_err(|Error| format!("Error creating output file : {}", Error))?;

    let Enumerations = Enumeration::Generate_code(LVGL_functions.Get_signatures());

    let Functions = Call::Generate_code(
        LVGL_functions.Get_type_tree(),
        LVGL_functions.Get_signatures(),
    )?;

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
        .map_err(|Error| format!("Error writing to output file : {}", Error))?;

    Format_rust(&Output_file_path)?;

    Ok(())
}
