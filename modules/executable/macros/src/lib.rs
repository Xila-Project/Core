use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Expr, Field, Fields, GenericParam, LitChar, LitStr, Type, parse_macro_input,
    spanned::Spanned,
};

fn is_bool_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == "bool")
            .unwrap_or(false),
        _ => false,
    }
}

// Added helper to detect NonZero types (e.g., NonZeroU32, NonZeroI64, NonZero, etc.)
fn is_nonzero_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string().starts_with("NonZero"))
            .unwrap_or(false),
        _ => false,
    }
}

fn is_str_reference_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => match reference.elem.as_ref() {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident == "str")
                .unwrap_or(false),
            _ => false,
        },
        _ => false,
    }
}

fn to_kebab_case(name: &str) -> String {
    name.replace('_', "-")
}

#[derive(Default)]
struct FieldConfig {
    positional: bool,
    flag: bool,
    short: Option<char>,
    long: Option<String>,
    default: Option<Expr>,
    legacy_default_used: bool,
}

fn parse_field_config(field: &Field) -> syn::Result<FieldConfig> {
    let mut config = FieldConfig::default();

    for attr in &field.attrs {
        if attr.path().is_ident("arg") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("positional") {
                    config.positional = true;
                    return Ok(());
                }

                if meta.path.is_ident("flag") {
                    config.flag = true;
                    return Ok(());
                }

                if meta.path.is_ident("short") {
                    let value: LitChar = meta.value()?.parse()?;
                    config.short = Some(value.value());
                    return Ok(());
                }

                if meta.path.is_ident("long") {
                    let value: LitStr = meta.value()?.parse()?;
                    config.long = Some(value.value());
                    return Ok(());
                }

                if meta.path.is_ident("default") {
                    let value: Expr = meta.value()?.parse()?;
                    config.default = Some(value);
                    return Ok(());
                }

                Err(meta.error(
                    "unsupported argument, expected one of: positional, flag, short, long, default",
                ))
            })?;
        }

        if attr.path().is_ident("default") {
            config.legacy_default_used = true;

            match &attr.meta {
                syn::Meta::List(list) => {
                    let value: Expr = syn::parse2(list.tokens.clone())?;
                    config.default = Some(value);
                }
                syn::Meta::NameValue(named) => {
                    config.default = Some(named.value.clone());
                }
                syn::Meta::Path(_) => {
                    return Err(syn::Error::new(
                        attr.span(),
                        "#[default] requires a value, e.g. #[default(4)]",
                    ));
                }
            }
        }
    }

    Ok(config)
}

#[proc_macro_derive(GetArgs, attributes(arg, default))]
pub fn derive_get_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let lifetime_params: Vec<_> = input
        .generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Lifetime(lifetime) => Some(lifetime.lifetime.clone()),
            _ => None,
        })
        .collect();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "GetArgs only works on structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "GetArgs only works on structs")
                .to_compile_error()
                .into();
        }
    };

    let mut initializers = Vec::new();
    let mut option_match_arms = Vec::new();
    let mut positional_match_arms = Vec::new();
    let mut required_checks = Vec::new();
    let mut field_builds = Vec::new();

    let mut positional_index = 0usize;

    for field in fields {
        let Some(field_name) = field.ident.as_ref() else {
            return syn::Error::new_spanned(field, "GetArgs only works on named fields")
                .to_compile_error()
                .into();
        };

        let field_type = &field.ty;
        let field_str = field_name.to_string();
        let config = match parse_field_config(field) {
            Ok(config) => config,
            Err(error) => return error.to_compile_error().into(),
        };

        let derived_long = to_kebab_case(&field_str);
        let derived_short = field_str
            .chars()
            .next()
            .expect("field name should never be empty");

        let is_option = !config.positional
            && (config.short.is_some()
                || config.long.is_some()
                || config.flag
                || config.default.is_some()
                || config.legacy_default_used);

        let is_required = config.default.is_none() && !config.flag;

        // Adjusted default value resolution to handle NonZero types cleanly
        let default_value = config.default.unwrap_or_else(|| {
            if config.flag {
                syn::parse_quote!(false)
            } else if is_nonzero_type(field_type) {
                // NonZero types don't implement Default. Use MIN (which is 1 for Unsigned)
                // as a safe implicit fallback value if a default wasn't provided.
                syn::parse_quote!(<#field_type>::MIN)
            } else {
                syn::parse_quote!(Default::default())
            }
        });

        let value_ident = format_ident!("__value_{}", field_name);
        let seen_ident = format_ident!("__seen_{}", field_name);

        if is_option {
            if config.flag && !is_bool_type(field_type) {
                return syn::Error::new_spanned(
                    field,
                    "#[arg(flag)] can only be used with bool fields",
                )
                .to_compile_error()
                .into();
            }

            let long_name = config.long.unwrap_or(derived_long);
            let short_name = config.short.unwrap_or(derived_short);

            let assign_value = if config.flag {
                quote! {
                    #value_ident = true;
                }
            } else if is_str_reference_type(field_type) {
                quote! {
                    let value = options
                        .value()
                        .map_err(|_| crate::Error::MissingPositionalArgument(#long_name))?;
                    #value_ident = value;
                }
            } else {
                quote! {
                    let value = options
                        .value()
                        .map_err(|_| crate::Error::MissingPositionalArgument(#long_name))?;
                    #value_ident = value.parse().map_err(|_| crate::Error::InvalidOption)?;
                }
            };

            initializers.push(quote! {
                let mut #value_ident: #field_type = #default_value;
                let mut #seen_ident = false;
            });

            option_match_arms.push(quote! {
                getargs::Arg::Short(#short_name) | getargs::Arg::Long(#long_name) => {
                    #assign_value
                    #seen_ident = true;
                }
            });

            if is_required {
                required_checks.push(quote! {
                    if !#seen_ident {
                        return Err(crate::Error::MissingPositionalArgument(#long_name));
                    }
                });
            }
        } else {
            let current_index = positional_index;
            positional_index += 1;

            initializers.push(quote! {
                let mut #value_ident: #field_type = #default_value;
                let mut #seen_ident = false;
            });

            let assign_positional = if is_str_reference_type(field_type) {
                quote! {
                    #value_ident = value;
                }
            } else {
                quote! {
                    #value_ident = value.parse().map_err(|_| crate::Error::InvalidOption)?;
                }
            };

            positional_match_arms.push(quote! {
                #current_index => {
                    #assign_positional
                    #seen_ident = true;
                }
            });

            if is_required {
                required_checks.push(quote! {
                    if !#seen_ident {
                        return Err(crate::Error::MissingPositionalArgument(#field_str));
                    }
                });
            }
        }

        field_builds.push(quote! { #field_name: #value_ident });
    }

    let positional_handler = if positional_match_arms.is_empty() {
        quote! {
            getargs::Arg::Positional(_) => {
                return Err(crate::Error::InvalidNumberOfArguments);
            }
        }
    } else {
        quote! {
            getargs::Arg::Positional(value) => {
                match __positional_index {
                    #(#positional_match_arms)*
                    _ => return Err(crate::Error::InvalidNumberOfArguments),
                }
                __positional_index += 1;
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn parse<'__getargs, I>(options: &mut getargs::Options<&'__getargs str, I>) -> core::result::Result<Self, crate::Error>
            where
                I: Iterator<Item = &'__getargs str>,
                #('__getargs: #lifetime_params,)*
            {
                #(#initializers)*
                let mut __positional_index = 0_usize;

                while let Some(argument) = options.next_arg().map_err(|_| crate::Error::InvalidOption)? {
                    match argument {
                        #(#option_match_arms)*
                        #positional_handler
                        _ => return Err(crate::Error::InvalidOption),
                    }
                }

                #(#required_checks)*

                Ok(Self {
                    #(#field_builds),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
