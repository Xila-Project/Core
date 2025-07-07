use std::fs;
use std::{fs::File, io::Write, path::Path};

use Bindings_utilities::Format::Format_identifier;

use quote::ToTokens;
use syn::{FnArg, ReturnType, Signature, Type};
use Bindings_utilities::Context::LVGL_context;
use Bindings_utilities::Format::Format_C;
use Bindings_utilities::Function::Split_inputs;

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
        "char" | "c_char" => "char".to_string(),
        "c_void" => "void".to_string(),
        "str" => "char *".to_string(),
        "*" => "*".to_string(),
        "const" => "const".to_string(),
        _ => Get_type_name(Type),
    }
}

pub fn Convert_type(Type: String) -> String {
    let type_value = Type.split_whitespace().collect::<Vec<_>>();

    let Type = type_value
        .into_iter()
        .filter(|x| *x != "mut" && *x != "core" && *x != "ffi" && *x != "::" && !x.is_empty())
        .rev()
        .collect::<Vec<_>>();

    let Type = Type
        .iter()
        .map(|x| Convert_fundamental_type(x))
        .collect::<Vec<_>>()
        .join(" ");

    let Type = Type
        .replace("Xila_graphics_object_t *", "Xila_graphics_object_t")
        .replace("Xila_graphics_object_t const *", "Xila_graphics_object_t");

    Type.replace("const Xila_graphics_object_t", "Xila_graphics_object_t")
}

fn Generate_function_signature(Signature: &Signature) -> String {
    let identifier = Get_function_name(&Signature.ident.to_string());

    let Inputs = Signature.inputs.iter().collect::<Vec<_>>();

    let (_, Inputs) = Split_inputs(&Inputs).unwrap();

    let mut Inputs = Inputs
        .iter()
        .map(|argument| match argument {
            syn::FnArg::Typed(pattern) => {
                let identifier = pattern.pat.to_token_stream().to_string();
                let type_value = Convert_type(pattern.ty.to_token_stream().to_string());
                format!("{type_value} {identifier}")
            }
            _ => panic!("Unsupported argument type"),
        })
        .collect::<Vec<_>>();

    if let ReturnType::Type(_, Type) = &Signature.output {
        let type_value = Convert_type(Type.to_token_stream().to_string());
        Inputs.push(format!("{type_value}* __result"));
    }

    format!("void {}({})", identifier, Inputs.join(", "))
}

fn Generate_function_declarations(Signatures: Vec<Signature>) -> String {
    let functions = Signatures
        .iter()
        .map(Generate_function_signature)
        .collect::<Vec<_>>();

    let Functions = functions.join(";\n");
    Functions + ";\n"
}

fn Generate_opaque_types(Structures: Vec<String>) -> String {
    let opaque_types = Structures
        .iter()
        .filter(|Type| Type.ends_with("dsc_t"))
        .map(|type_value| {
            format!(
                "typedef struct {{}} {};\n",
                type_value.replace("lv_", "Xila_graphics_"),
            )
        })
        .collect::<Vec<_>>();

    opaque_types.join("\n")
}

fn Generate_graphics_call(Signature: &Signature) -> String {
    let identifier = Get_enumerate_item(&Signature.ident.to_string());

    let Inputs = Signature.inputs.iter().collect::<Vec<_>>();

    let (_, Inputs) = Split_inputs(&Inputs).unwrap();

    let mut Inputs = Inputs
        .iter()
        .map(|argument| match argument {
            FnArg::Typed(pattern) => match &*pattern.ty {
                Type::Path(path) => {
                    if path.to_token_stream().to_string() == "lv_color_t"
                        || path.to_token_stream().to_string() == "lv_color32_t"
                        || path.to_token_stream().to_string() == "lv_color16_t"
                        || path.to_token_stream().to_string() == "lv_style_value_t"
                    {
                        format!("*(size_t*)&{}", pattern.pat.to_token_stream())
                    } else {
                        pattern.pat.to_token_stream().to_string()
                    }
                }
                Type::Ptr(_) => {
                    format!("(size_t){}", pattern.pat.to_token_stream())
                }
                Type => panic!("Unsupported argument type : {Type:?}"),
            },
            Receiver => panic!("Unsupported argument type : {Receiver:?}"),
        })
        .collect::<Vec<_>>();

    let Real_arguments_length = Inputs.len();

    for _ in Inputs.len()..7 {
        Inputs.push("0".to_string());
    }

    let Declaration = match &Signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, type_value) => {
            let type_value = Convert_type(type_value.to_token_stream().to_string());

            let Declaration = format!("{type_value} __result;");

            Some(Declaration)
        }
    };

    format!(
        "Xila_graphics_call({},{}, {}, {});\n",
        identifier,
        Inputs.join(", "),
        Real_arguments_length,
        Declaration
            .as_ref()
            .map(|_| "(void*)__result")
            .unwrap_or("NULL"),
    )
}

pub fn Generate_types(LVGL_functions: &LVGL_context) -> String {
    //Read to string
    let Includes = fs::read_to_string("./Build/Includes.h").unwrap();

    let Structures_name = LVGL_functions
        .Get_structures()
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>();

    let Opaque_types = Generate_opaque_types(Structures_name);

    let Types = fs::read_to_string("./Build/Types.h").unwrap();

    format!("{Includes}\n{Opaque_types}\n{Types}")
}

pub fn Generate_header(Output_file: &mut File, LVGL_functions: &LVGL_context) {
    Output_file
        .write_all(
            r#"
    #ifndef XILA_GRAPHICS_H
    #define XILA_GRAPHICS_H

    #ifdef __cplusplus
    extern "C" {
    #endif

    "#
            .as_bytes(),
        )
        .unwrap();

    Output_file
        .write_all(Generate_types(LVGL_functions).as_bytes())
        .expect("Error writing to bindings file");

    let Functions = Generate_function_declarations(LVGL_functions.Get_signatures());

    Output_file
        .write_all(Functions.as_bytes())
        .expect("Error writing to bindings file");

    Output_file
        .write_all(
            r#"
    #ifdef __cplusplus
        }
    #endif

    #endif
    "#
            .as_bytes(),
        )
        .unwrap();
}

pub fn Get_type_name(Type: &str) -> String {
    Format_identifier("Xila_graphics_", Type)
}

pub fn Get_function_name(Function_name: &str) -> String {
    Format_identifier("Xila_graphics_", Function_name)
}

pub fn Get_enumerate_item(Item: &str) -> String {
    Format_identifier("Xila_graphics_call_", Item)
}

pub fn Generate_code_enumeration(Signatures: Vec<Signature>) -> String {
    let mut signatures = Signatures.clone();

    signatures.sort_by_key(|x| x.ident.to_string().to_lowercase());

    let Function_calls = signatures
        .iter()
        .enumerate()
        .map(|(i, x)| format!("{} = {}", Get_enumerate_item(&x.ident.to_string()), i))
        .collect::<Vec<_>>()
        .join(",\n");

    format!("enum {{\n{Function_calls}\n}};\n")
}

pub fn Generate_C_function_definition(Signature: &Signature) -> String {
    let c_signature = Generate_function_signature(Signature);

    let Graphics_call = Generate_graphics_call(Signature);

    format!("{c_signature}\n{{\n{Graphics_call}\n}}\n")
}

pub fn Generate_source(Output_file: &mut File, Context: &LVGL_context) {
    Output_file
        .write_all("#include \"Xila_graphics.h\"\n".as_bytes())
        .unwrap();

    Output_file
        .write_all(Generate_code_enumeration(Context.Get_signatures()).as_bytes())
        .unwrap();

    let Prelude = fs::read_to_string("./Build/Prelude.c").unwrap();

    Output_file
        .write_all(Prelude.as_bytes())
        .expect("Error writing to bindings file");

    let Graphics_calls = Context
        .Get_signatures()
        .iter()
        .map(Generate_C_function_definition)
        .collect::<Vec<_>>()
        .join("\n");

    Output_file.write_all(Graphics_calls.as_bytes()).unwrap();
}

pub fn Generate(Output_path: &Path, LVGL_functions: &LVGL_context) -> Result<(), String> {
    let header_file_path = Output_path.join("Xila_graphics.h");
    let source_file_path = Output_path.join("Xila_graphics.c");

    let mut Header_file =
        File::create(&header_file_path).map_err(|_| "Error creating header file")?;
    let mut source_file =
        File::create(&source_file_path).map_err(|_| "Error creating source file")?;

    Generate_header(&mut Header_file, LVGL_functions);
    Generate_source(&mut source_file, LVGL_functions);

    Format_C(&header_file_path)?;
    Format_C(&source_file_path)?;

    Ok(())
}
