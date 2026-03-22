use std::{collections::HashMap, fs};

use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;

static TRANSLATION_PATH: Lazy<std::path::PathBuf> = Lazy::new(|| {
    let path = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .expect("CARGO_MANIFEST_DIR is not set");
    path.join("locales")
        .canonicalize()
        .expect("Failed to canonicalize path")
});

static LOCALE: Lazy<String> =
    Lazy::new(|| std::env::var("INTERNATIONALIZATION_LOCALE").unwrap_or_else(|_| "en".to_string()));

static FALLBACK_LOCALE: Lazy<String> = Lazy::new(|| {
    std::env::var("INTERNATIONALIZATION_FALLBACK").unwrap_or_else(|_| "en".to_string())
});

static TRANSLATION_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut generated_items = HashMap::new();

    // Load locale file
    let locale_file_path = TRANSLATION_PATH
        .clone()
        .join(format!("{}.json", LOCALE.to_lowercase()));
    if locale_file_path.exists() {
        match fs::read_to_string(&locale_file_path) {
            Ok(content) => match serde_json::from_str::<HashMap<String, String>>(&content) {
                Ok(translations) => {
                    for (key, value) in translations {
                        if !value.is_empty() {
                            generated_items.insert(key, value);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing JSON file {:?}: {}", locale_file_path, e);
                }
            },
            Err(e) => {
                eprintln!("Failed to read locale file {:?}: {}", locale_file_path, e);
            }
        }
    }

    // Load fallback file
    let fallback_file_path = TRANSLATION_PATH
        .clone()
        .join(format!("{}.json", FALLBACK_LOCALE.to_lowercase()));
    if fallback_file_path.exists() {
        match fs::read_to_string(&fallback_file_path) {
            Ok(content) => match serde_json::from_str::<HashMap<String, String>>(&content) {
                Ok(translations) => {
                    for (key, value) in translations {
                        generated_items.entry(key).or_insert(value);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error parsing fallback JSON file {:?}: {}",
                        fallback_file_path, e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read fallback locale file {:?}: {}",
                    fallback_file_path, e
                );
            }
        }
    }

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

    let value = TRANSLATION_MAP.get(identifier).cloned().unwrap_or_else(|| {
        panic!(
            "Translation for '{}' not found in locale or fallback (path: {:?})",
            identifier,
            TRANSLATION_PATH.clone()
        )
    });

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
