use bindings_utilities::{
    context::LvglContext, enumeration, file::write_token_stream_to_file, function::split_inputs,
};
use std::path::Path;

use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, ItemFn, ReturnType, Signature, TypePath};

use bindings_utilities::type_tree::TypeTree;

fn generate_conversion_for_argument(
    type_tree: &TypeTree,
    argument: &FnArg,
) -> Result<TokenStream, String> {
    match argument {
        FnArg::Typed(pattern) => {
            let identifier = &*pattern.pat;

            match &*pattern.ty {
                syn::Type::Ptr(type_value) => Ok(quote! {
                    let #identifier : #type_value = #identifier as _;
                }),
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
                            let #identifier = unsafe { core::mem::transmute::<u32, #path_string_identifier>(#identifier as u32) };
                        })
                    } else if path_string_stripped == "lv_color16_t" {
                        Ok(quote! {
                            let #identifier = unsafe { core::mem::transmute::<u16, #path_string_identifier>(#identifier as u16) };
                        })
                    } else if path_string_stripped == "lv_style_value_t" {
                        Ok(quote! {
                            let #identifier = #identifier as *mut lv_style_value_t;
                            let #identifier = unsafe { *#identifier };
                        })
                    } else if path_string_stripped == "usize" {
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
                    quote! {
                        let __result : *mut #type_value = __result as *mut _;
                    }
                }
                syn::Type::Path(r#type) => {
                    quote! {
                        let __result : *mut #r#type = __result as *mut _;
                    }
                }
                t => {
                    return Err(format!("Unsupported return type : {t:?}"));
                }
            };

            Ok(Some(quote! {
                #conversion

                unsafe {
                    *__result = __current_result;
                }
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
    type_tree: &TypeTree,
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
    let variant_identifier = enumeration::get_variant_identifier(&function.ident);

    // - Generate the return conversion if needed (let __result = __current_result;)
    let r#return = generate_conversion_for_output(&function.output)?;

    // - Generate the code for the function call (let __current_result = Function_identifier(arguments);)
    let function_call = if let Some(r#return) = &r#return {
        quote! {
            let __current_result = unsafe { #function_identifier(#(
                #call_arguments,
            )*) };

            #r#return
        }
    } else {
        quote! {
            unsafe {
                #function_identifier(#(
                    #call_arguments,
                )*);
            }
        }
    };

    Ok(quote! {
        FunctionCall::#variant_identifier => {
            // Check arguments count
            if __arguments_count != #arguments_count {
                return Err(Error::InvalidArgumentsCount);
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
    type_tree: &TypeTree,
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
            __task: TaskIdentifier,
            __function: FunctionCall,
            __argument_0: usize,
            __argument_1: usize,
            __argument_2: usize,
            __argument_3: usize,
            __argument_4: usize,
            __argument_5: usize,
            __argument_6: usize,
            __arguments_count: u8,
            __result: *mut core::ffi::c_void,
            __custom_data: *mut core::ffi::c_void,
        ) -> Result<()>
        {
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

fn generate_enumeration_impl() -> TokenStream {
    let pointers_offset_litteral = Literal::usize_unsuffixed(enumeration::POINTERS_OFFSET as usize);
    let lvgl_pointers_offset_litteral =
        Literal::usize_unsuffixed(enumeration::LVGL_POINTERS_OFFSET as usize);

    quote! {
        impl FunctionCall {
            fn get_bit(value: u32, index: u32) -> bool {
                // 1. Shift the bit at 'index' to the far right
                // 2. Use AND with 1 to isolate it
                // 3. Compare the result to 1
                (value >> index) & 1 == 1
            }

            pub fn is_function_argument_pointer(&self, index: usize) -> bool {
                let value = *self as u32;
                core::assert!(index < 8);
                Self::get_bit(value, index as u32 + #pointers_offset_litteral)
            }

            pub fn is_function_argument_lvgl_pointer(&self, index: usize) -> bool {
                let value = *self as u32;
                core::assert!(index < 8);
                Self::get_bit(value, index as u32 + #lvgl_pointers_offset_litteral)
            }

            pub fn is_function_return_pointer(&self) -> bool {
                let value = *self as u32;
                Self::get_bit(value, #pointers_offset_litteral + 8)
            }

            pub fn is_function_return_lvgl_pointer(&self) -> bool {
                let value = *self as u32;
                Self::get_bit(value, #lvgl_pointers_offset_litteral + 8)
            }

        }
    }
}

pub fn generate(output_path: &Path, context: &LvglContext) -> Result<(), String> {
    let output_file_path = output_path.join("bindings.rs");

    let enumerations = enumeration::generate_code(context.get_signatures());

    let enumerations_impl = generate_enumeration_impl();

    let functions = generate_code(
        context.get_type_tree(),
        context.get_signatures(),
        context.get_definitions(),
    )?;

    let token_stream = quote! {
        #enumerations

        #enumerations_impl

        #functions
    };

    write_token_stream_to_file(output_file_path, token_stream)?;

    Ok(())
}
