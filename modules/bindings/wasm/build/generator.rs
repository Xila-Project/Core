use std::fs;
use std::{fs::File, io::Write, path::Path};

use bindings_utilities::format::format_identifier;

use bindings_utilities::context::LvglContext;
use bindings_utilities::format::format_c;
use bindings_utilities::function::split_inputs;
use quote::ToTokens;
use syn::{FnArg, ReturnType, Signature, Type};

pub fn convert_fundamental_type(r#type: &str) -> String {
    match r#type {
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
        _ => get_type_name(r#type),
    }
}

pub fn convert_type(r#type: String) -> String {
    let type_value = r#type.split_whitespace().collect::<Vec<_>>();

    let r#type = type_value
        .into_iter()
        .filter(|x| *x != "mut" && *x != "core" && *x != "ffi" && *x != "::" && !x.is_empty())
        .rev()
        .collect::<Vec<_>>();

    let r#type = r#type
        .iter()
        .map(|x| convert_fundamental_type(x))
        .collect::<Vec<_>>()
        .join(" ");

    let r#type = r#type
        .replace("Xila_graphics_object_t *", "Xila_graphics_object_t")
        .replace("Xila_graphics_object_t const *", "Xila_graphics_object_t");

    r#type.replace("const Xila_graphics_object_t", "Xila_graphics_object_t")
}

fn generate_function_signature(signature: &Signature) -> String {
    let identifier = get_function_name(&signature.ident.to_string());

    let inputs = signature.inputs.iter().collect::<Vec<_>>();

    let (_, inputs) = split_inputs(&inputs).unwrap();

    let mut inputs = inputs
        .iter()
        .map(|argument| match argument {
            syn::FnArg::Typed(pattern) => {
                let identifier = pattern.pat.to_token_stream().to_string();
                let type_value = convert_type(pattern.ty.to_token_stream().to_string());
                format!("{type_value} {identifier}")
            }
            _ => panic!("Unsupported argument type"),
        })
        .collect::<Vec<_>>();

    if let ReturnType::Type(_, r#type) = &signature.output {
        let type_value = convert_type(r#type.to_token_stream().to_string());
        inputs.push(format!("{type_value}* __result"));
    }

    format!("void {}({})", identifier, inputs.join(", "))
}

fn generate_function_declarations(signatures: Vec<Signature>) -> String {
    let functions = signatures
        .iter()
        .map(generate_function_signature)
        .collect::<Vec<_>>();

    let functions = functions.join(";\n");
    functions + ";\n"
}

fn generate_opaque_types(structures: Vec<String>) -> String {
    let opaque_types = structures
        .iter()
        .filter(|r#type| r#type.ends_with("dsc_t"))
        .map(|type_value| {
            format!(
                "typedef struct {{}} {};\n",
                type_value.replace("lv_", "Xila_graphics_"),
            )
        })
        .collect::<Vec<_>>();

    opaque_types.join("\n")
}

fn generate_graphics_call(signature: &Signature) -> String {
    let identifier = get_enumerate_item(&signature.ident.to_string());

    let inputs = signature.inputs.iter().collect::<Vec<_>>();

    let (_, inputs) = split_inputs(&inputs).unwrap();

    let mut inputs = inputs
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
                r#type => panic!("Unsupported argument type : {type:?}"),
            },
            receiver => panic!("Unsupported argument type : {receiver:?}"),
        })
        .collect::<Vec<_>>();

    let real_arguments_length = inputs.len();

    for _ in inputs.len()..7 {
        inputs.push("0".to_string());
    }

    let declaration = match &signature.output {
        ReturnType::Default => None,
        ReturnType::Type(_, type_value) => {
            let type_value = convert_type(type_value.to_token_stream().to_string());

            let declaration = format!("{type_value} __result;");

            Some(declaration)
        }
    };

    format!(
        "Xila_graphics_call({},{}, {}, {});\n",
        identifier,
        inputs.join(", "),
        real_arguments_length,
        declaration
            .as_ref()
            .map(|_| "(void*)__result")
            .unwrap_or("NULL"),
    )
}

pub fn generate_types(lvgl_functions: &LvglContext) -> String {
    //Read to string
    let includes: String = fs::read_to_string("./build/includes.h").unwrap();

    let structures_name = lvgl_functions
        .get_structures()
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>();

    let opaque_types = generate_opaque_types(structures_name);

    let types = fs::read_to_string("./build/types.h").unwrap();

    format!("{includes}\n{opaque_types}\n{types}")
}

pub fn generate_header(output_file: &mut File, lvgl_functions: &LvglContext) {
    output_file
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

    output_file
        .write_all(generate_types(lvgl_functions).as_bytes())
        .expect("Error writing to bindings file");

    let functions = generate_function_declarations(lvgl_functions.get_signatures());

    output_file
        .write_all(functions.as_bytes())
        .expect("Error writing to bindings file");

    output_file
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

pub fn get_type_name(r#type: &str) -> String {
    format_identifier("Xila_graphics_", r#type)
}

pub fn get_function_name(function_name: &str) -> String {
    format_identifier("Xila_graphics_", function_name)
}

pub fn get_enumerate_item(item: &str) -> String {
    format_identifier("Xila_graphics_call_", item)
}

pub fn generate_code_enumeration(signatures: Vec<Signature>) -> String {
    let mut signatures = signatures.clone();

    signatures.sort_by_key(|x| x.ident.to_string().to_lowercase());

    let function_calls = signatures
        .iter()
        .enumerate()
        .map(|(i, x)| format!("{} = {}", get_enumerate_item(&x.ident.to_string()), i))
        .collect::<Vec<_>>()
        .join(",\n");

    format!("enum {{\n{function_calls}\n}};\n")
}

pub fn generate_c_function_definition(signature: &Signature) -> String {
    let c_signature = generate_function_signature(signature);

    let graphics_call = generate_graphics_call(signature);

    format!("{c_signature}\n{{\n{graphics_call}\n}}\n")
}

pub fn generate_source(output_file: &mut File, context: &LvglContext) {
    output_file
        .write_all("#include \"xila_graphics.h\"\n".as_bytes())
        .unwrap();

    output_file
        .write_all(generate_code_enumeration(context.get_signatures()).as_bytes())
        .unwrap();

    let prelude = fs::read_to_string("./build/prelude.c").unwrap();

    output_file
        .write_all(prelude.as_bytes())
        .expect("Error writing to bindings file");

    let graphics_calls = context
        .get_signatures()
        .iter()
        .map(generate_c_function_definition)
        .collect::<Vec<_>>()
        .join("\n");

    output_file.write_all(graphics_calls.as_bytes()).unwrap();
}

pub fn generate(output_path: &Path, lvgl_functions: &LvglContext) -> Result<(), String> {
    let header_file_path = output_path.join("xila_graphics.h");
    let source_file_path = output_path.join("xila_graphics.c");

    let mut header_file =
        File::create(&header_file_path).map_err(|_| "Error creating header file")?;
    let mut source_file =
        File::create(&source_file_path).map_err(|_| "Error creating source file")?;

    generate_header(&mut header_file, lvgl_functions);
    generate_source(&mut source_file, lvgl_functions);

    format_c(&header_file_path)?;
    format_c(&source_file_path)?;

    Ok(())
}
