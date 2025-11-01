fn main() {
    println!("cargo:rerun-if-env-changed=INTERNATIONALIZATION_LOCALE");
    println!("cargo:rerun-if-env-changed=INTERNATIONALIZATION_FALLBACK");
}
