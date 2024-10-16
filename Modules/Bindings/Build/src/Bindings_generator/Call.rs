use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, ReturnType, Signature, TypePath};

use super::Type_tree::Type_tree_type;

fn Generate_conversion_for_argument(Type_tree: &Type_tree_type, Argument: &FnArg) -> TokenStream {
    match Argument {
        FnArg::Typed(Pattern) => {
            let Identifier = &*Pattern.pat;
            match &*Pattern.ty {
                syn::Type::Ptr(Type) => {
                    quote! {
                        let #Identifier : #Type = __Environment.Convert_to_native_pointer(
                            #Identifier
                        ) as *mut _;
                    }
                }
                syn::Type::Path(Path) => {
                    let Path_string = Type_tree.Resolve(&Path.path);

                    let Path_string_stripped = Path_string.replace(" ", "");

                    let Path_string_identifier: TypePath = syn::parse_str(&Path_string).unwrap();

                    if Path_string_stripped == "bool" {
                        quote! {
                            let #Identifier = #Identifier != 0;
                        }
                    } else if Path_string_stripped == "lv_color_t" {
                        quote! {
                            let #Identifier = lv_color_t {
                                blue: #Identifier as u8,
                                green: (#Identifier >> 8) as u8,
                                red: (#Identifier >> 16) as u8,
                            };
                        }
                    } else if Path_string_stripped == "lv_color32_t" {
                        quote! {
                            let #Identifier = core::mem::transmute::<u32, #Path_string_identifier>(#Identifier as u32);
                        }
                    } else if Path_string_stripped == "lv_color16_t" {
                        quote! {
                            let #Identifier = core::mem::transmute::<u16, #Path_string_identifier>(#Identifier as u16);
                        }
                    } else if Path_string_stripped == "lv_style_value_t" {
                        quote! {
                            let #Identifier = #Identifier as *mut lv_style_value_t;
                            let #Identifier = *#Identifier;
                        }
                    } else if Path_string_stripped.starts_with("::core::option::Option<") {
                        quote! {
                            let #Identifier = if #Identifier == 0 {
                                core::option::Option::None
                            } else {
                                #[allow(clippy::missing_transmute_annotations)] // Too heavy to implement
                                core::option::Option::Some(core::mem::transmute(#Identifier))
                            };
                        }
                    } else if Path_string_stripped == "usize" {
                        quote! {}
                    } else {
                        quote! {
                            let #Identifier = #Identifier as #Path_string_identifier;
                        }
                    }
                }
                T => {
                    panic!("Unsupported type conversion : {:?}", T);
                }
            }
        }
        _ => {
            panic!("Unsupported argument type");
        }
    }
}

fn Generate_conversion_for_output(Return: &ReturnType) -> Option<TokenStream> {
    match Return {
        ReturnType::Type(_, Type) => {
            let Conversion = match &**Type {
                syn::Type::Ptr(_) => {
                    quote! {
                        let __Current_result = __Environment.Convert_to_WASM_pointer(
                            __Current_result
                        );

                        let __Result = __Result as *mut WASM_pointer_type;
                    }
                }
                syn::Type::Path(Type) => {
                    quote! {
                        let __Result = __Result as *mut #Type;
                    }
                }
                T => {
                    panic!("Unsupported type conversion : {:?}", T);
                }
            };

            Some(quote! {
                #Conversion

                *__Result = __Current_result;
            })
        }
        // If the return type is not specified, we don't need to convert it
        ReturnType::Default => None,
    }
}

fn Generate_from_signature(Type_tree: &Type_tree_type, Function: &Signature) -> TokenStream {
    let Conversion = Function
        .inputs
        .iter()
        .map(|Arguments| Generate_conversion_for_argument(Type_tree, Arguments))
        .collect::<Vec<_>>();

    let Assigns = Function
        .inputs
        .iter()
        .enumerate()
        .map(|(Index, x)| match x {
            FnArg::Typed(Pattern) => {
                let Identifier = format_ident!("__Argument_{}", Index);
                let Identifier_real = &*Pattern.pat;
                quote! {
                    let #Identifier_real = #Identifier;
                }
            }
            _ => {
                panic!("Unsupported argument type");
            }
        })
        .collect::<Vec<_>>();

    let Arguments = Function
        .inputs
        .iter()
        .map(|x| match x {
            FnArg::Typed(Pattern) => {
                let Identifier = &*Pattern.pat;
                quote! {
                    #Identifier
                }
            }
            _ => {
                panic!("Unsupported argument type");
            }
        })
        .collect::<Vec<_>>();

    let Arguments_count = proc_macro2::Literal::usize_unsuffixed(Arguments.len());

    let Identifier = &Function.ident;

    let Return = Generate_conversion_for_output(&Function.output);

    let Call = if let Some(Return) = &Return {
        quote! {
            let __Current_result = #Identifier(#(
                #Arguments,
            )*);

            #Return
        }
    } else {
        quote! {
            #Identifier(#(
                #Arguments,
            )*);
        }
    };

    quote! {
        Function_calls_type::#Identifier => {
            // Check arguments count
            if __Arguments_count != #Arguments_count {
                return;
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
            #Call
        }
    }
}

pub fn Generate_code(Type_tree: &Type_tree_type, Signatures: Vec<Signature>) -> TokenStream {
    let Functions = Signatures
        .iter()
        .map(|x| Generate_from_signature(Type_tree, x))
        .collect::<Vec<_>>();

    quote! {

        use super::lvgl::*;
        use Virtual_machine::{Environment_type, WASM_pointer_type, WASM_usize_type};

        #[allow(clippy::too_many_arguments)]
        pub unsafe fn Call_function(
            __Environment: Environment_type,
            __Function: Function_calls_type,
            __Argument_0: WASM_usize_type,
            __Argument_1: WASM_usize_type,
            __Argument_2: WASM_usize_type,
            __Argument_3: WASM_usize_type,
            __Argument_4: WASM_usize_type,
            __Argument_5: WASM_usize_type,
            __Argument_6: WASM_usize_type,
            __Arguments_count: WASM_usize_type,
            __Result: WASM_pointer_type,
        )
        {
            let __Result: *mut core::ffi::c_void  = __Environment.Convert_to_native_pointer(__Result);

            match __Function {
                #(
                    #Functions
                )*

            }

        }
    }
    .to_token_stream()
}
