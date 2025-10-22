use std::ops::Deref;
use std::path::Path;

use bindings_utilities::enumeration;
use bindings_utilities::file::write_token_stream_to_file;
use bindings_utilities::format::snake_to_upper_camel_case;

use bindings_utilities::context::LvglContext;
use bindings_utilities::function::{get_function_identifier, is_public_input};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, Ident, ReturnType, Signature, Type};

fn convert_type(mut ty: Type) -> Type {
    match &mut ty {
        Type::Path(path) => {
            if let Some(last_segment) = path.path.segments.last_mut() {
                let new_ident = match last_segment.ident.to_string().as_str() {
                    "lv_obj_t" | "_lv_obj_t" => format_ident!("Object"),
                    "lv_obj_class_t" => format_ident!("ObjectClass"),
                    "lv_obj_flag_t" => format_ident!("ObjectFlag"),
                    "lv_obj_point_transform_flag_t" => format_ident!("ObjectPointTransformFlag"),
                    "lv_result_t" => format_ident!("LvglResult"),
                    identifier => {
                        let ident = if identifier.starts_with("lv_") {
                            let ident = identifier.strip_prefix("lv_").unwrap_or(identifier);
                            let ident = ident.strip_suffix("_t").unwrap_or(ident);
                            snake_to_upper_camel_case(ident)
                        } else {
                            identifier.to_string()
                        };

                        Ident::new(&ident, last_segment.ident.span())
                    }
                };

                last_segment.ident = new_ident;
            }
        }
        Type::Ptr(pointer_type) => {
            *pointer_type.elem = convert_type(pointer_type.elem.deref().clone());
        }
        r#type => panic!("Unsupported argument type : {type:?}"),
    }
    ty
}

fn convert_function_argument(input: &&syn::FnArg) -> TokenStream {
    match input {
        FnArg::Typed(pattern_type) => {
            let pattern = &pattern_type.pat;
            let ty = convert_type(pattern_type.ty.deref().clone());

            quote! {
                #pattern : #ty
            }
        }
        receiver => panic!("Unsupported argument type : {receiver:?}"),
    }
}

fn get_passed_input(input: &&syn::FnArg) -> TokenStream {
    match input {
        FnArg::Typed(pattern_type) => match &*pattern_type.ty {
            Type::Path(path) => {
                let pattern = &pattern_type.pat;
                if path.to_token_stream().to_string() == "lv_color_t"
                    || path.to_token_stream().to_string() == "lv_color32_t"
                    || path.to_token_stream().to_string() == "lv_color16_t"
                    || path.to_token_stream().to_string() == "lv_style_value_t"
                {
                    quote! {
                        as_usize( #pattern )
                    }
                } else {
                    quote! {
                        #pattern as usize
                    }
                }
            }
            Type::Ptr(_) => {
                let pattern = &pattern_type.pat;
                quote! {
                    #pattern as usize
                }
            }
            r#type => panic!("Unsupported argument type : {type:?}"),
        },
        receiver => panic!("Unsupported argument type : {receiver:?}"),
    }
}

fn generate_xila_graphics_call(signature: &Signature) -> TokenStream {
    let filtered_inputs = signature
        .inputs
        .iter()
        .filter(is_public_input)
        .collect::<Vec<_>>();

    let enumeration_variant = enumeration::get_variant_identifier(&signature.ident);

    let mut passed_arguments = filtered_inputs
        .iter()
        .map(get_passed_input)
        .collect::<Vec<_>>();

    let remaining_arguments = 7 - passed_arguments.len();
    for _ in 0..remaining_arguments {
        passed_arguments.push(quote! { 0 });
    }

    let argument_count = filtered_inputs.len() as u8;

    let passed_result = if let ReturnType::Type(..) = &signature.output {
        quote! {
            __result_pointer as *mut _ as *mut core::ffi::c_void
        }
    } else {
        quote! { core::ptr::null_mut() }
    };

    quote! {
        xila_graphics_call(
                    crate::FunctionCall::#enumeration_variant,
                    #( #passed_arguments ),*,
                    #argument_count,
                    #passed_result
                )
    }
}

fn generate_c_function(signature: &Signature) -> TokenStream {
    let filtered_inputs = signature
        .inputs
        .iter()
        .filter(is_public_input)
        .collect::<Vec<_>>();

    let mut inputs = filtered_inputs
        .iter()
        .map(convert_function_argument)
        .collect::<Vec<_>>();

    let function_identifier = get_function_identifier("xila_graphics_", &signature.ident);

    if let ReturnType::Type(_, type_value) = &signature.output {
        let result_argument_type = convert_type(type_value.deref().clone());
        let result_argument = quote! {
            __result_pointer: *mut #result_argument_type
        };

        inputs.push(result_argument);
    }

    let generated_xila_graphics_call = generate_xila_graphics_call(signature);

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #function_identifier(
            #( #inputs ),*
        ) -> i32 {
            unsafe {
                #generated_xila_graphics_call
            }
        }
    }
}

fn generate_function(signature: &Signature) -> TokenStream {
    let function_identifier = get_function_identifier("", &signature.ident);

    let filtered_inputs = signature
        .inputs
        .iter()
        .filter(is_public_input)
        .collect::<Vec<_>>();

    let inputs = filtered_inputs
        .iter()
        .map(convert_function_argument)
        .collect::<Vec<_>>();

    let result_data_type = if let ReturnType::Type(_, type_value) = &signature.output {
        let type_value = convert_type(type_value.deref().clone());

        quote! {
            #type_value
        }
    } else {
        quote! { () }
    };

    let result_data_expression = quote! {
        let mut __result_data: #result_data_type = unsafe { core::mem::zeroed() };
        let __result_pointer = &mut __result_data as *mut _;
    };

    let generated_xila_graphics_call = generate_xila_graphics_call(signature);

    quote! {
        pub unsafe fn #function_identifier(
            #( #inputs ),*
        ) -> Result<#result_data_type> {

            #result_data_expression

            let __result_status =
            unsafe {
                #generated_xila_graphics_call
            };

            if __result_status == 0 {
                Ok(__result_data)
            } else {
                Err(__result_status)
            }

        }
    }
}

pub fn generate_enumeration(
    path: impl AsRef<Path>,
    lvgl_functions: &LvglContext,
) -> Result<(), String> {
    let token_stream = enumeration::generate_code(lvgl_functions.get_signatures().to_vec());

    write_token_stream_to_file(path, token_stream)?;

    Ok(())
}

pub fn generate_functions(
    path: impl AsRef<Path>,
    lvgl_functions: &LvglContext,
) -> Result<(), String> {
    let generated_functions = lvgl_functions
        .get_signatures()
        .iter()
        .map(generate_function)
        .collect::<Vec<_>>();

    let token_stream = quote! {
       use crate::prelude::*;

       #( #generated_functions )*
    };

    write_token_stream_to_file(path, token_stream)?;

    Ok(())
}

pub fn generate_c_abi_functions(
    path: impl AsRef<Path>,
    lvgl_functions: &LvglContext,
) -> Result<(), String> {
    let generated_c_abi_functions = lvgl_functions
        .get_signatures()
        .iter()
        .map(generate_c_function)
        .collect::<Vec<_>>();

    let token_stream = quote! {
       use crate::prelude::*;

        #( #generated_c_abi_functions )*
    };

    write_token_stream_to_file(path, token_stream)?;

    Ok(())
}
