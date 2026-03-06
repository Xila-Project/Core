use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, parse_macro_input};

#[proc_macro_derive(GetArgs, attributes(default))]
pub fn derive_get_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("GetArgs only works on structs with named fields"),
        },
        _ => panic!("GetArgs only works on structs"),
    };

    // Prepare variable initializers and match arms
    let mut initializers = Vec::new();
    let mut flag_arms = Vec::new();
    let mut positional_field = None;

    for field in fields {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_type = &field.ty;
        let field_str = field_name.to_string();
        let short_char = field_str.chars().next().unwrap();

        // Check for #[default(...)] attribute
        let mut default_val = quote! { Default::default() };
        let mut is_positional = true;

        for attr in &field.attrs {
            if attr.path().is_ident("default") {
                is_positional = false;
                if let Meta::List(list) = &attr.meta {
                    let tokens = &list.tokens;
                    default_val = quote! { #tokens };
                }
            }
        }

        if is_positional {
            positional_field = Some(field_name.clone());
            initializers.push(quote! { let mut #field_name: #field_type = ""; });
        } else {
            initializers.push(quote! { let mut #field_name: #field_type = #default_val; });
            flag_arms.push(quote! {
                getargs::Arg::Short(#short_char) | getargs::Arg::Long(#field_str) => {
                    let value = options.next_positional()
                        .ok_or(crate::Error::MissingPositionalArgument(#field_str))?;
                    #field_name = value.parse().map_err(|_| crate::Error::InvalidOption)?;
                }
            });
        }
    }

    let positional_logic = if let Some(p_field) = positional_field {
        quote! {
            getargs::Arg::Positional(p) => {
                if !#p_field.is_empty() { return Err(crate::Error::InvalidNumberOfArguments); }
                #p_field = p;
            }
        }
    } else {
        quote! { getargs::Arg::Positional(_) => return Err(crate::Error::InvalidOption), }
    };

    let field_names = fields.iter().map(|f| &f.ident);

    let expanded = quote! {
        impl #impl_generics #name #ty_generics {
            pub fn parse<I>(options: &mut getargs::Options<&'a str, I>) -> Result<Self, crate::Error>
            where I: Iterator<Item = &'a str>
            {
                #(#initializers)*

                while let Some(argument) = options.next_arg().map_err(|_| crate::Error::InvalidOption)? {
                    match argument {
                        #(#flag_arms)*
                        #positional_logic
                        _ => return Err(crate::Error::InvalidOption),
                    }
                }

                Ok(Self {
                    #(#field_names),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
