use std::ops::{BitAnd, BitOr, Not, Shl};

use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use syn::{FnArg, Ident, Signature};

use crate::utilities::{format::snake_ident_to_upper_camel, function::get_function_identifier};

pub fn get_variant_identifier(identifier: &Ident) -> Ident {
    let identifier = get_function_identifier("", identifier);
    snake_ident_to_upper_camel(&identifier)
}

pub fn generate_code(signatures: Vec<Signature>) -> TokenStream {
    let mut signatures = signatures.clone();

    signatures.sort_by_key(|x| x.ident.to_string().to_lowercase());

    let variants = &signatures
        .into_iter()
        .enumerate()
        .map(|(i, signature)| {
            let identifier = get_variant_identifier(&signature.ident);
            let value = Literal::usize_unsuffixed(i as usize);

            quote! { #identifier = #value }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u32)]
        pub enum FunctionCall {
            #(
                #variants,
            )*
        }
    }
    .to_token_stream()
}
