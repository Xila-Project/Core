#![allow(non_snake_case)]
use std::{env, process::Command};

fn main() -> Result<(), ()> {
    println!("cargo:rerun-if-changed=Tests/WASM_test/src/main.rs");
    println!("cargo:rerun-if-changed=Tests/WASM_test/Cargo.toml");

    // TODO : Add a check for test mode

    if env::var("RUSTUP_TOOLCHAIN").unwrap() == "esp" {
        println!("cargo:warning=Build of WASM tests are disabled for esp toolchain.");
        return Ok(());
    }

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir("Tests/WASM_test")
        .output()
        .unwrap();

    if !output.status.success() {
        println! {"cargo:warning=stderr: {}", String::from_utf8_lossy(&output.stderr)};
        println! {"cargo:warning=status: {}", output.status};
        return Err(());
    }

    Ok(())
}
