use quote::ToTokens;
use syn::FnArg;

pub fn split_inputs<'a>(
    inputs: &'a [&'a FnArg],
) -> Result<(&'a [&'a FnArg], &'a [&'a FnArg]), String> {
    let index = inputs
        .iter()
        .position(|argument| {
            if let FnArg::Typed(pattern) = argument {
                !pattern.pat.to_token_stream().to_string().starts_with("__")
            } else {
                false
            }
        })
        .unwrap_or(0);

    Ok(inputs.split_at(index))
}
