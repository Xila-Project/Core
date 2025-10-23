use std::{fs, path::Path};

use proc_macro2::TokenStream;
use quote::quote;

use crate::format::format_rust;

pub fn write_token_stream_to_file(
    path: impl AsRef<Path>,
    token_stream: TokenStream,
) -> Result<(), String> {
    let token_stream = quote! {
        /// Auto-generated file for Xila bindings WASM module
        /// Do not edit manually.

        #token_stream
    };

    fs::write(&path, token_stream.to_string())
        .map_err(|e| format!("Error writing to file: {}", e))?;

    format_rust(path)?;

    Ok(())
}
