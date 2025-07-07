#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{env, process::Command};

fn main() {
    if env::var("RUSTUP_TOOLCHAIN").unwrap().contains("esp") {
        println!("cargo:warning=Build of WASM tests are disabled for esp toolchain.");
        return;
    }

    let Output = Command::new("cargo")
        .current_dir("Tests/WASM_test")
        .arg("build")
        .arg("--release")
        .output()
        .unwrap();

    if !Output.status.success() {
        println! {"cargo:warning=stderr: {}", String::from_utf8_lossy(&Output.stderr)};
        println! {"cargo:warning=status: {}", Output.status};
    }
}
