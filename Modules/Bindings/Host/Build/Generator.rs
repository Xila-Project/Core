#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{fs::File, io::Write, path::Path};
use Bindings_utilities::{
    Context::LVGL_context, Enumeration, Format::Format_rust, Function::Split_inputs,
};

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, ItemFn, ReturnType, Signature, TypePath};

use Bindings_utilities::Type_tree::Type_tree_type;

fn Generate_conversion_for_argument(
    Type_tree: &Type_tree_type,
    Argument: &FnArg,
) -> Result<TokenStream, String> {
    match Argument {
        FnArg::Typed(Pattern) => {
            let Identifier = &*Pattern.pat;

            match &*Pattern.ty {
                syn::Type::Ptr(Type) => {
                    let Type_string = Type.elem.to_token_stream().to_string();

                    if Type_string == "lv_obj_t" {
                        Ok(quote! {

                            let #Identifier = __Pointer_table.Get_native_pointer(
                                __Task,
                                #Identifier.try_into().map_err(|_| Error_type::Invalid_pointer)?
                            )?;

                        })
                    } else {
                        Ok(quote! {
                            let #Identifier : #Type = Convert_to_native_pointer(
                                &__Environment,
                                #Identifier
                            )?;
                        })
                    }
                }
                syn::Type::Path(Path) => {
                    let Path_string = Type_tree.Resolve(&Path.path);

                    let Path_string_stripped = Path_string.replace(" ", "");

                    let Path_string_identifier: TypePath = syn::parse_str(&Path_string).unwrap();

                    if Path_string_stripped == "bool" {
                        Ok(quote! {
                            let #Identifier = #Identifier != 0;
                        })
                    } else if Path_string_stripped == "lv_color_t" {
                        Ok(quote! {
                            let #Identifier = lv_color_t {
                                blue: #Identifier as u8,
                                green: (#Identifier >> 8) as u8,
                                red: (#Identifier >> 16) as u8,
                            };
                        })
                    } else if Path_string_stripped == "lv_color32_t" {
                        Ok(quote! {
                            let #Identifier = core::mem::transmute::<u32, #Path_string_identifier>(#Identifier);
                        })
                    } else if Path_string_stripped == "lv_color16_t" {
                        Ok(quote! {
                            let #Identifier = core::mem::transmute::<u16, #Path_string_identifier>(#Identifier as u16);
                        })
                    } else if Path_string_stripped == "lv_style_value_t" {
                        Ok(quote! {
                            let #Identifier = #Identifier as *mut lv_style_value_t;
                            let #Identifier = *#Identifier;
                        })
                    } else if Path_string_stripped == "u32" {
                        Ok(quote! {})
                    } else {
                        Ok(quote! {
                            let #Identifier = #Identifier as #Path_string_identifier;
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
            let Conversion = match &**Type {
                syn::Type::Ptr(Type) => {
                    let Type_string = Type.elem.to_token_stream().to_string();

                    if Type_string == "lv_obj_t" {
                        quote! {

                            let __Result_2 : *mut u16 = Convert_to_native_pointer(&__Environment, __Result)?;

                            let __Result : *mut u16 = Convert_to_native_pointer(&__Environment, __Result)?;

                            let __Current_result = __Pointer_table.Insert(
                                __Task,
                                __Current_result as *mut core::ffi::c_void
                            )?;


                            *__Result = __Current_result;
                        }
                    } else if Type_string == "core :: ffi :: c_void" {
                        quote! {
                            let __Current_result = __Environment.Convert_to_WASM_pointer(
                                __Current_result
                            );

                            let __Result : *mut WASM_pointer_type = Convert_to_native_pointer(&__Environment, __Result)?;
                        }
                    } else {
                        quote! {
                            let __Current_result = __Environment.Convert_to_WASM_pointer(
                                __Current_result as *mut core::ffi::c_void
                            );

                            let __Result : *mut WASM_pointer_type = Convert_to_native_pointer(&__Environment, __Result)?;
                        }
                    }
                }
                syn::Type::Path(Type) => {
                    quote! {
                        let __Result : *mut #Type = Convert_to_native_pointer(&__Environment, __Result)?;
                    }
                }

                T => {
                    return Err(format!("Unsupported return type : {T:?}"));
                }
            };

            Ok(Some(quote! {
                #Conversion

                *__Result = __Current_result;

            }))
        }
        // If the return type is not specified, we don't need to convert it
        ReturnType::Default => Ok(None),
    }
}

fn Generate_assign(Index: usize, Argument: &FnArg) -> Result<TokenStream, String> {
    match Argument {
        FnArg::Typed(Pattern) => {
            let Identifier = format_ident!("__Argument_{}", Index);
            let Identifier_real = &*Pattern.pat;
            Ok(quote! {
                let #Identifier_real = #Identifier;
            })
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn Generate_call_argument(Argument: &FnArg) -> Result<TokenStream, String> {
    match Argument {
        FnArg::Typed(Pattern) => {
            let Identifier = &*Pattern.pat;
            Ok(quote! {
                #Identifier
            })
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn Generate_function_call(
    Type_tree: &Type_tree_type,
    Function: &Signature,
) -> Result<TokenStream, String> {
    // - Get the inputs
    let Inputs = Function.inputs.iter().collect::<Vec<_>>();

    let (Left_inputs, Right_inputs) = Split_inputs(&Inputs)?;

    // - Generate the assignation of the arguments to variables (let name = __Argument_X;)
    let Assigns = Right_inputs
        .iter()
        .enumerate()
        .map(|(i, Argument)| Generate_assign(i, Argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Generate the conversion of the arguments to the expected types (let name = name as Type;)
    let Conversion = Right_inputs
        .iter()
        .map(|Argument| Generate_conversion_for_argument(Type_tree, Argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Generate the order of the arguments in the function call (name, name, ...)
    let Call_arguments = Left_inputs
        .iter()
        .chain(Right_inputs.iter())
        .map(|Argument| Generate_call_argument(Argument))
        .collect::<Result<Vec<_>, _>>()?;

    // - Get the number of arguments
    let Arguments_count = Literal::usize_unsuffixed(Right_inputs.len());

    // - Get the function identifier
    let Function_identifier = &Function.ident;

    // - Generate the return conversion if needed (let __Result = __Current_result;)
    let Return = Generate_conversion_for_output(&Function.output)?;

    // - Generate the code for the function call (let __Current_result = Function_identifier(arguments);)
    let Function_call = if let Some(Return) = &Return {
        quote! {
            let __Current_result = #Function_identifier(#(
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
            if __Arguments_count != #Arguments_count {
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
    Type_tree: &Type_tree_type,
    Signatures: Vec<Signature>,
    Definitions: Vec<ItemFn>,
) -> Result<TokenStream, String> {
    let Functions_call = Signatures
        .iter()
        .map(|Signature| Generate_function_call(Type_tree, Signature))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #( #Definitions )*

        #[allow(unused_variables)]
        #[allow(clippy::too_many_arguments)]
        pub unsafe fn Call_function(
            __Environment: Environment_type,
            __Pointer_table: &mut Pointer_table_type,
            __Function: Function_calls_type,
            __Argument_0: WASM_usize_type,
            __Argument_1: WASM_usize_type,
            __Argument_2: WASM_usize_type,
            __Argument_3: WASM_usize_type,
            __Argument_4: WASM_usize_type,
            __Argument_5: WASM_usize_type,
            __Argument_6: WASM_usize_type,
            __Arguments_count: u8,
            __Result: WASM_pointer_type,
        ) -> Result_type<()>
        {
            let __Custom_data = __Environment.Get_or_initialize_custom_data().map_err(|_| Error_type::Failed_to_get_environment)?;
            let __Task = __Custom_data.Get_task_identifier();

            match __Function {
                #(
                    #Functions_call
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
    let mut Output_file = File::create(&Output_file_path)
        .map_err(|Error| format!("Error creating output file : {Error}"))?;

    let Enumerations = Enumeration::Generate_code(Context.Get_signatures());

    let Functions = Generate_code(
        Context.Get_type_tree(),
        Context.Get_signatures(),
        Context.Get_definitions(),
    )?;

    Output_file
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
