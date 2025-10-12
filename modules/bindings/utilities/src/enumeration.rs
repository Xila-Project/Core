use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use syn::Signature;

use crate::format::snake_ident_to_upper_camel;

pub fn generate_code(signatures: Vec<Signature>) -> TokenStream {
    let mut signatures = signatures.clone();

    signatures.sort_by_key(|x| x.ident.to_string().to_lowercase());

    let variants = &signatures
        .into_iter()
        .map(|x| snake_ident_to_upper_camel(&x.ident))
        .enumerate()
        .map(|(i, identifier)| {
            let i = Literal::usize_unsuffixed(i);
            quote! { #identifier = #i }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u16)]
        pub enum FunctionCall {
            #(
                #variants,
            )*
        }
    }
    .to_token_stream()
}
