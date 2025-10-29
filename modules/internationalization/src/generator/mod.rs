mod configuration;
mod extract;
mod parse;

use std::{ffi::CString, fs};

use crate::generator::parse::read_and_parse_locale_file;
use quote::quote;

pub use configuration::Configuration;
use syn::{LitCStr, LitStr};

pub fn generate_translation((key, value): (&String, &String)) -> proc_macro2::TokenStream {
    // Convert the key to a valid Rust identifier
    // Replace dots and other special characters with underscores
    let ident_name = key.replace('.', "__").replace('-', "_").to_lowercase();

    let value = LitStr::new(value, proc_macro2::Span::call_site());

    let c_string_value = LitCStr::new(
        CString::new(value.value())
            .expect("Failed to create CString")
            .as_c_str(),
        proc_macro2::Span::call_site(),
    );

    let ident = syn::Ident::new(&ident_name, proc_macro2::Span::call_site());

    quote! {
        #[allow(unused_macros)]
        macro_rules! #ident {
            () => { #value };
            (c) => { #c_string_value };
        }

        #[allow(unused_imports)]
        pub(crate) use #ident;

    }
}

pub fn generate_translations(configuration: &Configuration) -> Result<(), String> {
    let Configuration {
        input_path: path,
        locale,
        fallback,
        output_path,
    } = configuration;

    let path = path
        .canonicalize()
        .map_err(|err| format!("Failed to canonicalize path {:?}: {}", path, err))?;

    println!("cargo:rerun-if-env-changed=INTERNATIONALIZATION_LOCALE");
    println!("cargo:rerun-if-changed={}", path.display());

    // Read and parse the TOML file
    let toml_value = match read_and_parse_locale_file(&path) {
        Ok(value) => value,
        Err(err_msg) => {
            return Err(err_msg);
        }
    };
    // Extract translations for the specified locale with fallback
    let translations = extract::extract_translations(&toml_value, locale, fallback);

    // Generate the static strings
    let mut generated_items = translations
        .iter()
        .map(generate_translation)
        .collect::<Vec<_>>();

    generated_items.push(quote! {
        const __INTERNATIONALIZATION_LOCALE: &'static str = #locale;
        const __INTERNATIONALIZATION_FALLBACK: &'static str = #fallback;
    });

    let expanded = quote! {
        #(#generated_items)*
    };

    fs::write(output_path, expanded.to_string()).map_err(|err| {
        format!(
            "Failed to write generated translations to {:?}: {}",
            output_path, err
        )
    })?;

    Ok(())
}
