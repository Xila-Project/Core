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
    let Type = Type.split_whitespace().collect::<Vec<_>>();

    let mut Type = Type
        .into_iter()
        .filter(|x| *x != "mut" && *x != "core" && *x != "ffi" && *x != "::" && !x.is_empty())
        .collect::<Vec<_>>();

    while Type.first() == Some(&"*") {
        let String = Type.remove(0);
        Type.push(String);
    }

    let Type = Type
        .iter()
        .map(|x| Convert_fundamental_type(x))
        .collect::<Vec<_>>()
        .join(" ");

    let Type = Type.replace("Xila_graphics_object_t *", "Xila_graphics_object_t");

    Type.replace("const Xila_graphics_object_t", "Xila_graphics_object_t")
}

fn Generate_function_signature(Signature: &Signature) -> String {
    let Identifier = Get_function_name(&Signature.ident.to_string());

    let Inputs = Signature.inputs.iter().collect::<Vec<_>>();

    let (_, Inputs) = Split_inputs(&Inputs).unwrap();

    let mut Inputs = Inputs
        .iter()
        .map(|Argument| match Argument {
            syn::FnArg::Typed(Pattern) => {
                let Identifier = Pattern.pat.to_token_stream().to_string();
                let Type = Convert_type(Pattern.ty.to_token_stream().to_string());
                format!("{} {}", Type, Identifier)
            }
            _ => panic!("Unsupported argument type"),
        })
        .collect::<Vec<_>>();

    if let ReturnType::Type(_, Type) = &Signature.output {
        let Type = Convert_type(Type.to_token_stream().to_string());
        Inputs.push(format!("{}* __Result", Type));
    }

    format!("void {}({})", Identifier, Inputs.join(", "))
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
    let Identifier = Get_enumerate_item(&Signature.ident.to_string());

    let Inputs = Signature.inputs.iter().collect::<Vec<_>>();

    let (_, Inputs) = Split_inputs(&Inputs).unwrap();

    let mut Inputs = Inputs
        .iter()
        .map(|Argument| match Argument {
            FnArg::Typed(Pattern) => match &*Pattern.ty {
                Type::Path(Path) => {
                    if Path.to_token_stream().to_string() == "lv_color_t"
                        || Path.to_token_stream().to_string() == "lv_color32_t"
                        || Path.to_token_stream().to_string() == "lv_color16_t"
                        || Path.to_token_stream().to_string() == "lv_style_value_t"
                    {
                        format!("*(size_t*)&{}", Pattern.pat.to_token_stream())
                    } else {
                        Pattern.pat.to_token_stream().to_string()
                    }
                }
                Type::Ptr(_) => {
                    format!("(size_t){}", Pattern.pat.to_token_stream())
                }
                Type => panic!("Unsupported argument type : {:?}", Type),
            },
            Receiver => panic!("Unsupported argument type : {:?}", Receiver),
        })
        .collect::<Vec<_>>();

    let Real_arguments_length = Inputs.len();

    for _ in Inputs.len()..7 {
        Inputs.push("0".to_string());
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
        "Xila_graphics_call({},{}, {}, {});\n",
        Identifier,
        Inputs.join(", "),
        Real_arguments_length,
        Declaration
            .as_ref()
            .map(|_| "(void*)__Result")
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

    format!("{}\n{}\n{}", Includes, Opaque_types, Types)
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

pub fn Generate_code_enumeration(Signature: Vec<Signature>) -> String {
    let Function_calls = Signature
        .iter()
        .map(|x| Get_enumerate_item(&x.ident.to_string()))
        .collect::<Vec<_>>()
        .join(",\n");

    format!("enum {{\n{}\n}};\n", Function_calls)
}

pub fn Generate_C_function_definition(Signature: &Signature) -> String {
    let C_signature = Generate_function_signature(Signature);

    let Graphics_call = Generate_graphics_call(Signature);

    format!("{}\n{{\n{}\n}}\n", C_signature, Graphics_call)
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
    let Header_file_path = Output_path.join("Xila_graphics.h");
    let Source_file_path = Output_path.join("Xila_graphics.c");

    let mut Header_file =
        File::create(&Header_file_path).map_err(|_| "Error creating header file")?;
    let mut Source_file =
        File::create(&Source_file_path).map_err(|_| "Error creating source file")?;

    Generate_header(&mut Header_file, LVGL_functions);
    Generate_source(&mut Source_file, LVGL_functions);

    Format_C(&Header_file_path)?;
    Format_C(&Source_file_path)?;

    Ok(())
}
