use std::{env, path::Path};

mod guest;
mod host;
mod utilities;

pub fn main() {
    let out_path = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_path);

    if env::var("CARGO_FEATURE_GUEST").is_ok() {
        guest::generate(out_path);
    }

    if env::var("CARGO_FEATURE_HOST").is_ok() {
        host::generate(out_path).expect("Error generating host bindings");
    }
}
