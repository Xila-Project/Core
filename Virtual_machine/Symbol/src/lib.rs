#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, Ident, ItemFn, PatIdent, PatType, Result, Type,
};

#[derive(Clone)]
struct Argument_type {
    Parsed_argument: FnArg,
}

impl Argument_type {
    pub fn New(Parsed: FnArg) -> Self {
        Self {
            Parsed_argument: Parsed,
        }
    }

    pub fn Is_self(&self) -> bool {
        match &self.Parsed_argument {
            FnArg::Receiver(_) => true,
            _ => false,
        }
    }

    pub fn Get_name(&self) -> Option<Ident> {
        match &self.Parsed_argument {
            FnArg::Typed(PatType { pat, .. }) => match &**pat {
                syn::Pat::Ident(ident) => Some(ident.ident.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn Get_type(&self) -> Option<Type> {
        match &self.Parsed_argument {
            FnArg::Typed(ty) => Some(*ty.ty.clone()),
            _ => None,
        }
    }

    pub fn To_token_stream(&self) -> proc_macro2::TokenStream {
        let Parsed_argument = &self.Parsed_argument;
        quote! {
            #Parsed_argument
        }
    }
}

#[derive(Debug)]
enum WASM_argument_type {
    i32,
    i64,
    f32,
    f64,
    External_reference,
    String,
    Buffer(bool),
}

struct Function_type {
    Parsed_function: ItemFn,
}

impl Function_type {
    pub fn New(Parsed: ItemFn) -> Self {
        Self {
            Parsed_function: Parsed,
        }
    }

    pub fn Get_name(&self) -> Ident {
        self.Parsed_function.sig.ident.clone()
    }

    pub fn Get_binding_name(&self) -> Ident {
        format_ident!("{}_binding_function", self.Get_name())
    }

    fn Convert_type(mut Type: Type) -> (WASM_argument_type) {
        match &mut Type {
            Type::Path(Type_path) => {
                let Identifier = Type_path.path.segments.first().unwrap().ident.to_string();
                match Identifier.as_str() {
                    "i32" => return WASM_argument_type::i32,
                    "i64" => return WASM_argument_type::i64,
                    "f32" => return WASM_argument_type::f32,
                    "f64" => return WASM_argument_type::f64,
                    "String" => return WASM_argument_type::String,
                    "str" => return WASM_argument_type::String,
                    _ => {
                        println!("Identifier {:?}", Identifier);
                    }
                }
            }
            Type::Array(Type_array) => {
                let Type = Self::Convert_type(*Type_array.elem.clone());
                return WASM_argument_type::Array(false);
            }
            Type::Reference(Type_reference) => {
                let Type = Self::Convert_type(*Type_reference.elem.clone());
                if Type == WASM_argument_type::
                return WASM_argument_type::External_reference;
            }
            _ => {
                println!("Type {:?}", Type);
            }
        }
    }

    fn Convert_argument(mut Argument: FnArg) -> FnArg {
        let mut Pattern = match &mut Argument {
            FnArg::Typed(Pattern) => Pattern,
            _ => return Argument,
        };

        match Pattern.ty.as_mut() {
            _ => {
                println!("Pattern.ty {:?}", Pattern.ty);
            }
        }

        return Argument;
    }

    pub fn Get_binding_arguments(&self) -> proc_macro2::TokenStream {
        let Arguments: Vec<FnArg> = self
            .Parsed_function
            .sig
            .inputs
            .clone()
            .into_iter()
            .map(Self::Convert_argument)
            .collect();

        // - Append the arguments to the TokenStream
        let mut TokenStream = quote!(); // - The first argument is the execution environment of the virtual machine
        for Argument in Arguments {
            let T = quote! {
                #Argument
            };
            TokenStream.extend(T);
        }
        print!("TokenStream {:?}", TokenStream);
        TokenStream
    }

    pub fn Get_binding_body(&self) -> proc_macro2::TokenStream {
        let Name = self.Get_name();

        quote! {
            {
                #Name (a, b)
            }
        }
    }

    pub fn Get_declaration(&self) -> proc_macro2::TokenStream {
        let Binding_name = self.Get_binding_name();
        let Binding_declaration_name = format_ident!("{}_binding_declaration", self.Get_name());
        let Name = self.Get_name();
        let Name_str = Name.to_string();

        quote! {
            pub const #Binding_declaration_name  : super::Native_symbol_type = super::Native_symbol_type {
                symbol: concat!(#Name_str, "\0").as_ptr() as *const i8,
                func_ptr: super::#Binding_name as *mut _,
                signature: concat!("(ii)i", "\0").as_ptr() as *const i8,
                attachment: std::ptr::null_mut(),
            }
        }
    }

    pub fn Get_binding(&self) -> proc_macro::TokenStream {
        let Parsed_function = &self.Parsed_function;

        let Binding_name = self.Get_binding_name();
        let Binding_body = self.Get_binding_body();
        let Binding_arguments = self.Get_binding_arguments();

        let Declaration = self.Get_declaration();

        let Binding = quote! {
            #Parsed_function

            #[no_mangle]
            pub extern "C" fn #Binding_name (v : wasm_exec_env_t, #Binding_arguments) {
                #Binding_body
            }

            #Declaration;
        };

        proc_macro::TokenStream::from(Binding)
    }
}

#[proc_macro_attribute]
pub fn Use_native_function(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let e = Function_type::New(parse_macro_input!(item as ItemFn));

    proc_macro::TokenStream::from(e.Get_binding())
}
