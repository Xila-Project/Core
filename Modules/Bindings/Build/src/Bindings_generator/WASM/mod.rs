use std::{fs::File, io::Write, path::Path};

use super::Format::Format_C;

use super::Functions::LVGL_functions_type;
use quote::ToTokens;
use syn::{ReturnType, Signature};

pub fn Convert_fundamental_type(Type: &str) -> String {
    match Type {
        "bool" => "bool".to_string(),
        "u8" => "uint8_t".to_string(),
        "u16" => "uint16_t".to_string(),
        "u32" => "uint32_t".to_string(),
        "u64" => "uint64_t".to_string(),
        "i8" => "int8_t".to_string(),
        "i16" => "int16_t".to_string(),
        "i32" => "int32_t".to_string(),
        "i64" => "int64_t".to_string(),
        "f32" => "float".to_string(),
        "f64" => "double".to_string(),
        "usize" => "size_t".to_string(),
        "isize" => "ssize_t".to_string(),
        "char" => "char".to_string(),
        "str" => "char *".to_string(),
        _ => Type.replace("lv_", "Xila_graphics_"),
    }
}

pub fn Convert_type(Type: String) -> String {
    let Type = Type.to_string();

    let mut Type = Type.replace("mut", "");

    while Type.starts_with("*") {
        Type = Type
            .strip_prefix("*")
            .map(|x| format!("{} *", x))
            .unwrap_or(Type);

        Type = Type.trim().to_string();
    }

    let Type_components = Type.split_whitespace().collect::<Vec<_>>();

    let Type = Type_components
        .iter()
        .map(|x| Convert_fundamental_type(x))
        .collect::<Vec<_>>()
        .join(" ");

    let Type = Type.replace("core :: ffi :: c_char", "char");
    let Type = Type.replace("core :: ffi :: c_void", "char");

    Type.to_string()
}

fn Generate_function_signature(Signature: &Signature) -> String {
    let Identifier = Signature.ident.to_string();

    let Identifier = Identifier.replace("lv_", "Xila_graphics_");

    let Arguments = Signature
        .inputs
        .iter()
        .map(|Argument| match Argument {
            syn::FnArg::Typed(Pattern) => {
                let Identifier = Pattern.pat.to_token_stream().to_string();
                let Type = Convert_type(Pattern.ty.to_token_stream().to_string());
                format!("{} {}", Type, Identifier)
            }
            _ => panic!("Unsupported argument type"),
        })
        .collect::<Vec<_>>()
        .join(", ");

    let Return = match &Signature.output {
        ReturnType::Default => "void".to_string(),
        ReturnType::Type(_, Type) => Convert_type(Type.to_token_stream().to_string()),
    };

    format!("{} {}({})", Return, Identifier, Arguments)
}

fn Generate_function_declarations(Signatures: Vec<Signature>) -> String {
    let Functions = Signatures
        .iter()
        .map(Generate_function_signature)
        .collect::<Vec<_>>();

    let Functions = Functions.join(";\n");
    Functions + ";\n"
}

fn Generate_opaque_types(Structures: Vec<String>) -> String {
    let Opaque_types = Structures
        .iter()
        .filter(|Type| Type.ends_with("dsc_t"))
        .map(|Type| {
            format!(
                "typedef struct {{}} {};\n",
                Type.replace("lv_", "Xila_graphics_"),
            )
        })
        .collect::<Vec<_>>();

    Opaque_types.join("\n")
}

fn Generate_graphics_call(Signature: &Signature) -> String {
    let Identifier = Signature
        .ident
        .to_string()
        .replacen("lv_", "Xila_graphics_call_", 1);

    let mut Arguments = Signature
        .inputs
        .iter()
        .map(|Argument| match Argument {
            syn::FnArg::Typed(Pattern) => {
                if Pattern.ty.to_token_stream().to_string() == "lv_color_t"
                    || Pattern.ty.to_token_stream().to_string() == "lv_color32_t"
                    || Pattern.ty.to_token_stream().to_string() == "lv_color16_t"
                    || Pattern.ty.to_token_stream().to_string() == "lv_style_value_t"
                {
                    format!("*(size_t*)&{}", Pattern.pat.to_token_stream())
                } else {
                    Pattern.pat.to_token_stream().to_string()
                }
            }
            _ => panic!("Unsupported argument type"),
        })
        .collect::<Vec<_>>();

    let Real_arguments_length = Arguments.len();

    for _ in Arguments.len()..7 {
        Arguments.push("0".to_string());
    }

    let Declaration = match &Signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, Type) => {
            let Type = Convert_type(Type.to_token_stream().to_string());

            let Declaration = format!("{} __Result;", Type);

            Some(Declaration)
        }
    };

    format!(
        "{}\nXila_graphics_call({},{}, {}, {});\n{}",
        Declaration
            .as_ref()
            .map(|String| String.as_str())
            .unwrap_or(""),
        Identifier,
        Arguments.join(", "),
        Real_arguments_length,
        Declaration
            .as_ref()
            .map(|_| "(void*)&__Result")
            .unwrap_or("NULL"),
        Declaration
            .as_ref()
            .map(|_| "return __Result;")
            .unwrap_or("")
    )
}

fn Generate_function_definitions(Signatures: Vec<Signature>) -> String {
    let Functions = Signatures
        .iter()
        .map(|Signature| {
            let C_signature = Generate_function_signature(Signature);

            let Call = Generate_graphics_call(Signature);

            format!("{} {{\n    {}\n}}", C_signature, Call)
        })
        .collect::<Vec<_>>();

    Functions.join("\n")
}

pub fn Generate_header(Output_file: &mut File, LVGL_functions: &LVGL_functions_type) {
    Output_file
        .write_all(include_str!("Includes.h").as_bytes())
        .expect("Error writing to bindings file");

    let Opaque_types = Generate_opaque_types(LVGL_functions.Get_structures().clone());

    Output_file
        .write_all(Opaque_types.as_bytes())
        .expect("Error writing to bindings file");

    Output_file
        .write_all(include_str!("Types.h").as_bytes())
        .expect("Error writing to bindings file");

    let Functions = Generate_function_declarations(LVGL_functions.Get_signatures());

    Output_file
        .write_all(Functions.as_bytes())
        .expect("Error writing to bindings file");
}

pub fn Generate_code_enumeration(Signature: Vec<Signature>) -> String {
    let Function_calls = Signature
        .iter()
        .map(|x| {
            x.ident
                .to_string()
                .replacen("lv_", "Xila_graphics_call_", 1)
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "typedef enum {{\n{}\n}} Function_calls_type;\n",
        Function_calls
    )
}

pub fn Generate_source(Output_file: &mut File, LVGL_functions: &LVGL_functions_type) {
    Output_file
        .write_all(include_str!("Includes.c").as_bytes())
        .expect("Error writing to bindings file");

    Output_file
        .write_all(Generate_code_enumeration(LVGL_functions.Get_signatures()).as_bytes())
        .expect("Error writing to bindings file");

    Output_file
        .write_all(include_str!("Functions.c").as_bytes())
        .expect("Error writing to bindings file");

    let Functions = Generate_function_definitions(LVGL_functions.Get_signatures());

    Output_file
        .write_all(Functions.as_bytes())
        .expect("Error writing to bindings file");
}

pub fn Generate(Output_path: &Path, LVGL_functions: &LVGL_functions_type) -> Result<(), String> {
    let Source_file_path = Output_path.join("Xila_graphics.c");
    let Header_file_path = Output_path.join("Xila_graphics.h");

    let mut Header_file =
        File::create(&Header_file_path).map_err(|_| "Error creating header file")?;
    Generate_header(&mut Header_file, LVGL_functions);
    Format_C(&Header_file_path)?;

    let mut Source_file =
        File::create(&Source_file_path).map_err(|_| "Error creating source file")?;
    Generate_source(&mut Source_file, LVGL_functions);
    Format_C(&Source_file_path)?;

    Ok(())
}
