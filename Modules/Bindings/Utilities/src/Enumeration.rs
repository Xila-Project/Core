use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::Signature;

pub fn Generate_code(Signatures: Vec<Signature>) -> TokenStream {
    let mut Signatures = Signatures.clone();

    Signatures.sort_by_key(|x| x.ident.to_string().to_lowercase());

    let Variants = &Signatures
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            let Identifier = &x.ident;
            let i = Literal::usize_unsuffixed(i);
            quote! { #Identifier = #i }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u16)]
        pub enum Function_calls_type {
            #(
                #Variants,
            )*
        }
    }
    .to_token_stream()
}
