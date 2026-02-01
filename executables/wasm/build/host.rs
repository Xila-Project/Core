use crate::utilities::{
    self, context::LvglContext, enumeration, file::write_token_stream_to_file,
    function::split_inputs, type_tree::TypeTree,
};
use std::{env, path::Path};

use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, ItemFn, ReturnType, Signature, TypePath, visit::Visit};

fn generate_conversion_for_output(r#return: &ReturnType) -> Result<Option<TokenStream>, String> {
    match r#return {
        ReturnType::Type(_, r#type) => {
            let conversion = match &**r#type {
                syn::Type::Ptr(type_value) => {
                    let type_string = type_value.elem.to_token_stream().to_string();

                    if type_string == "lv_obj_t" {
                        quote! {


                            let __result : *mut u16 =  translate_to_native_pointer(&__environment, __result)? ;

                            let __current_result = __translation_map.insert(
                                __task,
                                __current_result as *mut core::ffi::c_void
                            )?;

                        }
                    } else if type_string == "core :: ffi :: c_void" {
                        quote! {
                            let __current_result =  __environment.convert_to_wasm_pointer(
                                __current_result
                            ) ;

                            let __result : *mut WasmPointer =  translate_to_native_pointer(&__environment, __result)? ;
                        }
                    } else {
                        quote! {
                            let __current_result =  __environment.convert_to_wasm_pointer(
                                __current_result as *mut core::ffi::c_void
                            ) ;

                            let __result : *mut WasmPointer =  translate_to_native_pointer(&__environment, __result)? ;
                        }
                    }
                }
                syn::Type::Path(r#type) => {
                    quote! {
                        let __result : *mut #r#type =  translate_to_native_pointer(&__environment, __result)? ;
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

fn generate_conversion_for_argument(
    index: usize,
    type_tree: &TypeTree,
    argument: &FnArg,
) -> Result<TokenStream, String> {
    let argument_identifier = format_ident!("__argument_{}", index);

    match argument {
        FnArg::Typed(pattern) => {
            let identifier = &*pattern.pat;

            match &*pattern.ty {
                syn::Type::Ptr(type_value) => {
                    let type_string = type_value.elem.to_token_stream().to_string();

                    if type_string == "lv_obj_t" {
                        Ok(quote! {

                            let #identifier = __translation_map.get_native_pointer(
                                __task,
                                #argument_identifier.try_into().map_err(|_| Error::InvalidPointer)?
                            )?;

                        })
                    } else {
                        Ok(quote! {
                            let #identifier : #type_value =  translate_to_native_pointer(
                                &__environment,
                                #argument_identifier
                            )? ;
                        })
                    }
                }
                syn::Type::Path(path) => {
                    let path_string = type_tree.resolve(&path.path);

                    let path_string_stripped = path_string.replace(" ", "");

                    let path_string_identifier: TypePath = syn::parse_str(&path_string).unwrap();

                    if path_string_stripped == "bool" {
                        Ok(quote! {
                            let #identifier = #argument_identifier != 0;
                        })
                    } else if path_string_stripped == "lv_color_t" {
                        Ok(quote! {
                            let #identifier = lv_color_t {
                                blue: #argument_identifier as u8,
                                green: (#argument_identifier >> 8) as u8,
                                red: (#argument_identifier >> 16) as u8,
                            };
                        })
                    } else if path_string_stripped == "lv_color32_t" {
                        Ok(quote! {
                            let #identifier = core::mem::transmute::<u32, #path_string_identifier>(#argument_identifier);
                        })
                    } else if path_string_stripped == "lv_color16_t" {
                        Ok(quote! {
                            let #identifier = core::mem::transmute::<u16, #path_string_identifier>(#argument_identifier as u16);
                        })
                    } else if path_string_stripped == "lv_style_value_t" {
                        Ok(quote! {
                            let #identifier = #argument_identifier as *mut lv_style_value_t;
                            let #identifier =  *#identifier ;
                        })
                    } else if path_string_stripped == "u32" {
                        Ok(quote! {
                            let #identifier = #argument_identifier as u32;
                        })
                    } else {
                        Ok(quote! {
                            let #identifier = #argument_identifier as #path_string_identifier;
                        })
                    }
                }
                t => Err(format!("Unsupported type conversion : {t:?}")),
            }
        }
        _ => Err("Unsupported argument type".to_string()),
    }
}

fn generate_assign(index: usize, argument: &FnArg) -> Result<TokenStream, String> {
    match argument {
        FnArg::Typed(pattern) => {
            let identifier = format_ident!("__argument_{}", index);
            let identifier_real = &*pattern.pat;
            Ok(quote! {
                let #identifier_real = FromUsize::from_usize(#identifier);
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
        .map(|(i, argument)| generate_conversion_for_argument(i, type_tree, argument))
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
            __environment: Environment,
            __translation_map: TranslationMap,
            __task: TaskIdentifier,
            __function: FunctionCall,
            __argument_0: WasmUsize,
            __argument_1: WasmUsize,
            __argument_2: WasmUsize,
            __argument_3: WasmUsize,
            __argument_4: WasmUsize,
            __argument_5: WasmUsize,
            __argument_6: WasmUsize,
            __arguments_count: u8,
            __result: WasmPointer,
        ) -> Result<()>
        {
            use xila::graphics::lvgl::*;
            use crate::host::bindings::graphics::{cast::{FromUsize, ToUsize}, error::{Error, Result},
            translate_to_native_pointer
        };

            let result = match __function {
                #(
                    #functions_call
                )*

            };

            Ok(result)
        }
    }
    .to_token_stream())
}

pub fn generate_inner(output_path: &Path, context: &LvglContext) -> Result<(), String> {
    let output_file_path = output_path.join("bindings.rs");

    let enumerations = enumeration::generate_code(context.get_signatures());

    let functions = generate_code(
        &context.get_type_tree(),
        context.get_signatures(),
        context.get_definitions(),
    )?;

    let token_stream = quote! {
        use crate::host::{
            bindings::graphics::TranslationMap,
            virtual_machine::{
                Environment, WasmPointer, WasmUsize
            }
        };

        #enumerations

        #functions
    };

    write_token_stream_to_file(output_file_path, token_stream)?;

    Ok(())
}

pub fn generate(out_path: &Path) -> Result<(), ()> {
    let input = lvgl_rust_sys::_bindgen_raw_src();
    let parsed_input = syn::parse_file(input).expect("Error parsing input file");

    let mut context = LvglContext::default();
    context.set_function_filtering(Some(LvglContext::filter_function));
    context.visit_file(&parsed_input);
    context.set_function_filtering(None);
    context.visit_file(&syn::parse2(utilities::additional::get()).unwrap());

    println!(
        "cargo:warning=bindings generation in {}",
        out_path.display()
    );

    generate_inner(out_path, &context).expect("Error generating native bindings");

    Ok(())
}
