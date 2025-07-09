use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::Signature;

pub fn generate_code(Signatures: Vec<Signature>) -> TokenStream {
    let mut signatures = Signatures.clone();

    signatures.sort_by_key(|x| x.ident.to_string().to_lowercase());

    let variants = &signatures
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            let identifier = &x.ident;
            let i = Literal::usize_unsuffixed(i);
            quote! { #identifier = #i }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u16)]
        pub enum Function_calls_type {
            #(
                #variants,
            )*
        }
    }
    .to_token_stream()
}
