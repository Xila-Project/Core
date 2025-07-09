#![allow(non_camel_case_types)]

use bindings_utilities::{
    context::LVGL_context, enumeration, format::format_rust, function::split_inputs,
};
use std::{fs::File, io::Write, path::Path};

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, ItemFn, ReturnType, Signature, TypePath};

use bindings_utilities::type_tree::Type_tree_type;

fn generate_conversion_for_argument(
    type_tree: &Type_tree_type,
    argument: &FnArg,
) -> Result<TokenStream, String> {
    match argument {
        FnArg::Typed(pattern) => {
            let identifier = &*pattern.pat;

            match &*pattern.ty {
                syn::Type::Ptr(type_value) => {
                    let type_string = type_value.elem.to_token_stream().to_string();

                    if type_string == "lv_obj_t" {
                        Ok(quote! {

                            let #identifier = __pointer_table.get_native_pointer(
                                __task,
                                #identifier.try_into().map_err(|_| Error_type::Invalid_pointer)?
                            )?;

                        })
                    } else {
                        Ok(quote! {
                            let #identifier : #type_value = convert_to_native_pointer(
                                &__environment,
                                #identifier
                            )?;
                        })
                    }
                }
                syn::Type::Path(path) => {
                    let path_string = type_tree.resolve(&path.path);

                    let path_string_stripped = path_string.replace(" ", "");

                    let path_string_identifier: TypePath = syn::parse_str(&path_string).unwrap();

                    if path_string_stripped == "bool" {
                        Ok(quote! {
                            let #identifier = #identifier != 0;
                        })
                    } else if path_string_stripped == "lv_color_t" {
                        Ok(quote! {
                            let #identifier = lv_color_t {
                                blue: #identifier as u8,
                                green: (#identifier >> 8) as u8,
                                red: (#identifier >> 16) as u8,
                            };
                        })
                    } else if path_string_stripped == "lv_color32_t" {
                        Ok(quote! {
                            let #identifier = core::mem::transmute::<u32, #path_string_identifier>(#identifier);
                        })
                    } else if path_string_stripped == "lv_color16_t" {
                        Ok(quote! {
                            let #identifier = core::mem::transmute::<u16, #path_string_identifier>(#identifier as u16);
                        })
                    } else if path_string_stripped == "lv_style_value_t" {
                        Ok(quote! {
                            let #identifier = #identifier as *mut lv_style_value_t;
                            let #identifier = *#identifier;
                        })
                    } else if path_string_stripped == "u32" {
                        Ok(quote! {})
                    } else {
                        Ok(quote! {
                            let #identifier = #identifier as #path_string_identifier;
                        })
                    }
                }
                t => Err(format!("Unsupported type conversion : {t:?}")),
            }
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn generate_conversion_for_output(r#return: &ReturnType) -> Result<Option<TokenStream>, String> {
    match r#return {
        ReturnType::Type(_, r#type) => {
            let conversion = match &**r#type {
                syn::Type::Ptr(type_value) => {
                    let type_string = type_value.elem.to_token_stream().to_string();

                    if type_string == "lv_obj_t" {
                        quote! {

                            let __result_2 : *mut u16 = convert_to_native_pointer(&__environment, __result)?;

                            let __result : *mut u16 = convert_to_native_pointer(&__environment, __result)?;

                            let __current_result = __pointer_table.insert(
                                __task,
                                __current_result as *mut core::ffi::c_void
                            )?;


                            *__result = __current_result;
                        }
                    } else if type_string == "core :: ffi :: c_void" {
                        quote! {
                            let __current_result = __environment.convert_to_wasm_pointer(
                                __current_result
                            );

                            let __result : *mut WASM_pointer_type = convert_to_native_pointer(&__environment, __result)?;
                        }
                    } else {
                        quote! {
                            let __current_result = __environment.convert_to_wasm_pointer(
                                __current_result as *mut core::ffi::c_void
                            );

                            let __result : *mut WASM_pointer_type = convert_to_native_pointer(&__environment, __result)?;
                        }
                    }
                }
                syn::Type::Path(r#type) => {
                    quote! {
                        let __result : *mut #r#type = convert_to_native_pointer(&__environment, __result)?;
                    }
                }

                t => {
                    return Err(format!("Unsupported return type : {t:?}"));
                }
            };

            Ok(Some(quote! {
                #conversion

                *__result = __current_result;

            }))
        }
        // If the return type is not specified, we don't need to convert it
        ReturnType::Default => Ok(None),
    }
}

fn generate_assign(index: usize, argument: &FnArg) -> Result<TokenStream, String> {
    match argument {
        FnArg::Typed(pattern) => {
            let identifier = format_ident!("__argument_{}", index);
            let identifier_real = &*pattern.pat;
            Ok(quote! {
                let #identifier_real = #identifier;
            })
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn generate_call_argument(argument: &FnArg) -> Result<TokenStream, String> {
    match argument {
        FnArg::Typed(pattern) => {
            let identifier = &*pattern.pat;
            Ok(quote! {
                #identifier
            })
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn generate_function_call(
    type_tree: &Type_tree_type,
    function: &Signature,
) -> Result<TokenStream, String> {
    // - Get the inputs
    let inputs = function.inputs.iter().collect::<Vec<_>>();

    let (left_inputs, right_inputs) = split_inputs(&inputs)?;

    // - Generate the assignation of the arguments to variables (let name = __Argument_X;)
    let assigns = right_inputs
        .iter()
        .enumerate()
        .map(|(i, argument)| generate_assign(i, argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Generate the conversion of the arguments to the expected types (let name = name as Type;)
    let conversion = right_inputs
        .iter()
        .map(|argument| generate_conversion_for_argument(type_tree, argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Generate the order of the arguments in the function call (name, name, ...)
    let call_arguments = left_inputs
        .iter()
        .chain(right_inputs.iter())
        .map(|argument| generate_call_argument(argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Get the number of arguments
    let arguments_count = Literal::usize_unsuffixed(right_inputs.len());

    // - Get the function identifier
    let function_identifier = &function.ident;

    // - Generate the return conversion if needed (let __result = __current_result;)
    let r#return = generate_conversion_for_output(&function.output)?;

    // - Generate the code for the function call (let __current_result = Function_identifier(arguments);)
    let function_call = if let Some(r#return) = &r#return {
        quote! {
            let __current_result = #function_identifier(#(
                #call_arguments,
            )*);

            #r#return
        }
    } else {
        quote! {
            #function_identifier(#(
                #call_arguments,
            )*);
        }
    };

    Ok(quote! {
        Function_calls_type::#function_identifier => {
            // Check arguments count
            if __arguments_count != #arguments_count {
                return Err(Error_type::Invalid_arguments_count);
            }
            // Assign arguments
            #(
                #assigns
            )*
            // Convert arguments
            #(
                #conversion
            )*

            // Call function
            #function_call
        }
    })
}

pub fn generate_code(
    type_tree: &Type_tree_type,
    signatures: Vec<Signature>,
    definitions: Vec<ItemFn>,
) -> Result<TokenStream, String> {
    let functions_call = signatures
        .iter()
        .map(|signature| generate_function_call(type_tree, signature))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #( #definitions )*

        #[allow(unused_variables)]
        #[allow(clippy::too_many_arguments)]
        pub unsafe fn call_function(
            __environment: Environment_type,
            __pointer_table: &mut Pointer_table_type,
            __function: Function_calls_type,
            __argument_0: WASM_usize_type,
            __argument_1: WASM_usize_type,
            __argument_2: WASM_usize_type,
            __argument_3: WASM_usize_type,
            __argument_4: WASM_usize_type,
            __argument_5: WASM_usize_type,
            __argument_6: WASM_usize_type,
            __arguments_count: u8,
            __result: WASM_pointer_type,
        ) -> Result_type<()>
        {
            let __custom_data = __environment.get_or_initialize_custom_data().map_err(|_| Error_type::Failed_to_get_environment)?;
            let __task = __custom_data.get_task_identifier();

            match __function {
                #(
                    #functions_call
                )*

            }

            Ok(())

        }
    }
    .to_token_stream())
}

pub fn generate(output_path: &Path, context: &LVGL_context) -> Result<(), String> {
    // Open the output fileoutput_path
    let output_file_path = output_path.join("Bindings.rs");
    let mut output_file = File::create(&output_file_path)
        .map_err(|error| format!("Error creating output file : {error}"))?;

    let enumerations = enumeration::generate_code(context.get_signatures());

    let functions = generate_code(
        context.get_type_tree(),
        context.get_signatures(),
        context.get_definitions(),
    )?;

    output_file
        .write_all(
            quote! {
                    #enumerations

                    #functions

            }
            .to_string()
            .as_bytes(),
        )
        .map_err(|error| format!("Error writing to output file : {error}"))?;

    format_rust(&output_file_path)?;

    Ok(())
}
