#![allow(non_camel_case_types)]

use std::{fs::File, io::Write, path::Path};
use Bindings_utilities::{
    Context::LVGL_context, Enumeration, Format::Format_rust, Function::Split_inputs,
};

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, ItemFn, ReturnType, Signature, TypePath};

use Bindings_utilities::Type_tree::Type_tree_type;

fn Generate_conversion_for_argument(
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

                            let #identifier = __pointer_table.Get_native_pointer(
                                __task,
                                #identifier.try_into().map_err(|_| Error_type::Invalid_pointer)?
                            )?;

                        })
                    } else {
                        Ok(quote! {
                            let #identifier : #type_value = Convert_to_native_pointer(
                                &__environment,
                                #identifier
                            )?;
                        })
                    }
                }
                syn::Type::Path(Path) => {
                    let path_string = type_tree.Resolve(&Path.path);

                    let Path_string_stripped = path_string.replace(" ", "");

                    let Path_string_identifier: TypePath = syn::parse_str(&path_string).unwrap();

                    if Path_string_stripped == "bool" {
                        Ok(quote! {
                            let #identifier = #identifier != 0;
                        })
                    } else if Path_string_stripped == "lv_color_t" {
                        Ok(quote! {
                            let #identifier = lv_color_t {
                                blue: #identifier as u8,
                                green: (#identifier >> 8) as u8,
                                red: (#identifier >> 16) as u8,
                            };
                        })
                    } else if Path_string_stripped == "lv_color32_t" {
                        Ok(quote! {
                            let #identifier = core::mem::transmute::<u32, #Path_string_identifier>(#identifier);
                        })
                    } else if Path_string_stripped == "lv_color16_t" {
                        Ok(quote! {
                            let #identifier = core::mem::transmute::<u16, #Path_string_identifier>(#identifier as u16);
                        })
                    } else if Path_string_stripped == "lv_style_value_t" {
                        Ok(quote! {
                            let #identifier = #identifier as *mut lv_style_value_t;
                            let #identifier = *#identifier;
                        })
                    } else if Path_string_stripped == "u32" {
                        Ok(quote! {})
                    } else {
                        Ok(quote! {
                            let #identifier = #identifier as #Path_string_identifier;
                        })
                    }
                }
                T => Err(format!("Unsupported type conversion : {T:?}")),
            }
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn Generate_conversion_for_output(Return: &ReturnType) -> Result<Option<TokenStream>, String> {
    match Return {
        ReturnType::Type(_, Type) => {
            let conversion = match &**Type {
                syn::Type::Ptr(type_value) => {
                    let type_string = type_value.elem.to_token_stream().to_string();

                    if type_string == "lv_obj_t" {
                        quote! {

                            let __result_2 : *mut u16 = Convert_to_native_pointer(&__environment, __result)?;

                            let __result : *mut u16 = Convert_to_native_pointer(&__environment, __result)?;

                            let __current_result = __pointer_table.Insert(
                                __task,
                                __current_result as *mut core::ffi::c_void
                            )?;


                            *__result = __current_result;
                        }
                    } else if type_string == "core :: ffi :: c_void" {
                        quote! {
                            let __current_result = __environment.Convert_to_WASM_pointer(
                                __current_result
                            );

                            let __result : *mut WASM_pointer_type = Convert_to_native_pointer(&__environment, __result)?;
                        }
                    } else {
                        quote! {
                            let __current_result = __environment.Convert_to_WASM_pointer(
                                __current_result as *mut core::ffi::c_void
                            );

                            let __result : *mut WASM_pointer_type = Convert_to_native_pointer(&__environment, __result)?;
                        }
                    }
                }
                syn::Type::Path(Type) => {
                    quote! {
                        let __result : *mut #Type = Convert_to_native_pointer(&__environment, __result)?;
                    }
                }

                T => {
                    return Err(format!("Unsupported return type : {T:?}"));
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

fn Generate_assign(Index: usize, Argument: &FnArg) -> Result<TokenStream, String> {
    match Argument {
        FnArg::Typed(pattern) => {
            let identifier = format_ident!("__argument_{}", Index);
            let identifier_real = &*pattern.pat;
            Ok(quote! {
                let #identifier_real = #identifier;
            })
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn Generate_call_argument(Argument: &FnArg) -> Result<TokenStream, String> {
    match Argument {
        FnArg::Typed(pattern) => {
            let identifier = &*pattern.pat;
            Ok(quote! {
                #identifier
            })
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn Generate_function_call(
    type_tree: &Type_tree_type,
    function: &Signature,
) -> Result<TokenStream, String> {
    // - Get the inputs
    let Inputs = function.inputs.iter().collect::<Vec<_>>();

    let (Left_inputs, Right_inputs) = Split_inputs(&Inputs)?;

    // - Generate the assignation of the arguments to variables (let name = __Argument_X;)
    let Assigns = Right_inputs
        .iter()
        .enumerate()
        .map(|(i, argument)| Generate_assign(i, argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Generate the conversion of the arguments to the expected types (let name = name as Type;)
    let Conversion = Right_inputs
        .iter()
        .map(|argument| Generate_conversion_for_argument(type_tree, argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Generate the order of the arguments in the function call (name, name, ...)
    let Call_arguments = Left_inputs
        .iter()
        .chain(Right_inputs.iter())
        .map(|argument| Generate_call_argument(argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Get the number of arguments
    let Arguments_count = Literal::usize_unsuffixed(Right_inputs.len());

    // - Get the function identifier
    let Function_identifier = &function.ident;

    // - Generate the return conversion if needed (let __result = __current_result;)
    let Return = Generate_conversion_for_output(&function.output)?;

    // - Generate the code for the function call (let __current_result = Function_identifier(arguments);)
    let Function_call = if let Some(Return) = &Return {
        quote! {
            let __current_result = #Function_identifier(#(
                #Call_arguments,
            )*);

            #Return
        }
    } else {
        quote! {
            #Function_identifier(#(
                #Call_arguments,
            )*);
        }
    };

    Ok(quote! {
        Function_calls_type::#Function_identifier => {
            // Check arguments count
            if __arguments_count != #Arguments_count {
                return Err(Error_type::Invalid_arguments_count);
            }
            // Assign arguments
            #(
                #Assigns
            )*
            // Convert arguments
            #(
                #Conversion
            )*

            // Call function
            #Function_call
        }
    })
}

pub fn Generate_code(
    type_tree: &Type_tree_type,
    signatures: Vec<Signature>,
    definitions: Vec<ItemFn>,
) -> Result<TokenStream, String> {
    let functions_call = signatures
        .iter()
        .map(|signature| Generate_function_call(type_tree, signature))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #( #definitions )*

        #[allow(unused_variables)]
        #[allow(clippy::too_many_arguments)]
        pub unsafe fn Call_function(
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
            let __custom_data = __environment.Get_or_initialize_custom_data().map_err(|_| Error_type::Failed_to_get_environment)?;
            let __task = __custom_data.Get_task_identifier();

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

pub fn Generate(Output_path: &Path, Context: &LVGL_context) -> Result<(), String> {
    // Open the output file
    let Output_file_path = Output_path.join("Bindings.rs");
    let mut output_file = File::create(&Output_file_path)
        .map_err(|error| format!("Error creating output file : {error}"))?;

    let Enumerations = Enumeration::Generate_code(Context.Get_signatures());

    let Functions = Generate_code(
        Context.Get_type_tree(),
        Context.Get_signatures(),
        Context.Get_definitions(),
    )?;

    output_file
        .write_all(
            quote! {
                    #Enumerations

                    #Functions

            }
            .to_string()
            .as_bytes(),
        )
        .map_err(|Error| format!("Error writing to output file : {Error}"))?;

    Format_rust(&Output_file_path)?;

    Ok(())
}
