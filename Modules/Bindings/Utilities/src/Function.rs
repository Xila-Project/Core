use quote::ToTokens;
use syn::FnArg;

pub fn Split_inputs<'a>(
    Inputs: &'a [&'a FnArg],
) -> Result<(&'a [&'a FnArg], &'a [&'a FnArg]), String> {
    let Index = Inputs
        .iter()
        .position(|Argument| {
            if let FnArg::Typed(Pattern) = Argument {
                !Pattern.pat.to_token_stream().to_string().starts_with("__")
            } else {
                false
            }
        })
        .unwrap_or(0);

    Ok(Inputs.split_at(Index))
}
