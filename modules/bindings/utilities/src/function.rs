use syn::{FnArg, Ident, Pat};

pub fn is_public_input(input: &&FnArg) -> bool {
    // If input ident start with two underscores, consider it private
    if let FnArg::Typed(pat_type) = input
        && let Pat::Ident(pat_ident) = &*pat_type.pat
    {
        return !pat_ident.ident.to_string().starts_with("__");
    }
    true
}

pub fn split_inputs<'a>(
    inputs: &'a [&'a FnArg],
) -> Result<(&'a [&'a FnArg], &'a [&'a FnArg]), String> {
    let index = inputs.iter().position(is_public_input).unwrap_or(0);

    Ok(inputs.split_at(index))
}
