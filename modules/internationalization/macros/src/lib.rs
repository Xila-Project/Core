mod file;
mod parse;

use std::{collections::HashMap, fs};

use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;

use crate::{file::filter_files, parse::PoParser};

static TRANSLATION_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let path = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .expect("CARGO_MANIFEST_DIR is not set");
    let path = path.join("locales");

    let locale = std::env::var("INTERNATIONALIZATION_LOCALE").unwrap_or("en".to_string());
    let fallback = std::env::var("INTERNATIONALIZATION_FALLBACK").unwrap_or("en".to_string());

    let path = path.canonicalize().expect("Failed to canonicalize path");

    let locale_directory_path = path.join(locale.to_lowercase());

    let po_files = fs::read_dir(&locale_directory_path)
        .expect("Failed to read locale directory")
        .filter_map(filter_files);

    let mut generated_items = HashMap::new();

    po_files.for_each(|content| {
        PoParser::new(&content).for_each(|res| match res {
            Ok((msgid, msgstr)) => {
                if !msgstr.is_empty() {
                    generated_items.insert(msgid, msgstr);
                }
            }
            Err(e) => {
                eprintln!("Error parsing PO file: {}", e);
            }
        });
    });

    let fallback_directory_path = path.join(fallback.to_lowercase());

    // open file
    let po_files = fs::read_dir(&fallback_directory_path)
        .map_err(|err| {
            format!(
                "Failed to read fallback locale directory {:?}: {}",
                &fallback_directory_path, err
            )
        })
        .expect("Failed to read fallback locale directory")
        .filter_map(filter_files);

    po_files.for_each(|content| {
        PoParser::new(&content).for_each(|res| match res {
            Ok((msgid, msgstr)) => {
                generated_items.entry(msgid).or_insert(msgstr);
            }
            Err(e) => {
                eprintln!("Error parsing PO file: {}", e);
            }
        });
    });

    generated_items
});

#[proc_macro]
pub fn translate(input: TokenStream) -> TokenStream {
    let input = input.to_string();

    let identifier = input.trim();
    let (c, identifier) = if let Some(s) = identifier.strip_prefix("c\"") {
        (true, s)
    } else {
        (false, identifier.strip_prefix("\"").unwrap_or(identifier))
    };

    let identifier = identifier.strip_suffix("\"").unwrap_or(identifier);

    let value = TRANSLATION_MAP
        .get(identifier)
        .cloned()
        .unwrap_or_else(|| panic!("Translation for '{}' not found", identifier));

    let value = if c {
        let c_string_value = syn::LitCStr::new(
            std::ffi::CString::new(value)
                .expect("Failed to create CString")
                .as_c_str(),
            proc_macro2::Span::call_site(),
        );
        quote! { #c_string_value }
    } else {
        let value = syn::LitStr::new(&value, proc_macro2::Span::call_site());
        quote! { #value }
    };

    value.into()
}
