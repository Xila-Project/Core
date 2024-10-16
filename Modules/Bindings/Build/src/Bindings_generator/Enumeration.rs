use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Signature;

pub fn Generate_code(Signatures: Vec<Signature>) -> TokenStream {
    let Variants = &Signatures.into_iter().map(|x| x.ident).collect::<Vec<_>>();

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr(u32)]
        pub enum Function_calls_type {
            #(
                #Variants,
            )*
        }
    }
    .to_token_stream()
}
