#!/bin/bash
# Execute this file with: `source Export.sh`

clear && echo "Setting up environment variables for Rust cross-compilation"

# Remove all aliases
{
unalias build_linux
unalias build_windows
unalias build_esp32 
unalias build_esp32_s3 
unalias run_linux
unalias run_windows 
unalias run_esp32
unalias run_esp32_s3
unalias test_linux
unalias test_windows
unalias test_esp32
unalias test_esp32_s3
unalias check_linux
unalias check_windows
unalias check_esp32
unalias check_esp32_s3
unalias clean
unalias format
} &> /dev/null

# Variables

Linux_environment_variables=""
Linux_target="--target x86_64-unknown-linux-gnu"
Windows_target="--target x86_64-pc-windows-gnu"

ESP32_environment_variables="MCU=esp32"
ESP32_target="--target xtensa-esp32-espidf -Z build-std=std,panic_abort"
ESP32_S3_environment_variables="MCU=esp32s3"
ESP32_S3_target="--target xtensa-esp32s3-espidf -Z build-std=std,panic_abort"

Cargo="cargo"
Cargo_esp="cargo +esp"
Clear="clear &&"

# Aliases
alias build_linux="$Clear $Linux_environment_variables $Cargo build $Linux_target"
alias run_linux="$Clear $Linux_environment_variables $Cargo run $Linux_target"
alias test_linux="$Clear RUST_MIN_STACK=8388608 $Linux_environment_variables $Cargo test $Linux_target"
alias check_linux="$Clear $Linux_environment_variables $Cargo clippy $Linux_target"

alias build_windows="$Clear $Cargo build $Windows_target"
alias run_windows="$Clear $Cargo run $Windows_target"
alias test_windows="$Clear $Cargo test $Windows_target"
alias check_windows="$Clear $Cargo check $Windows_target"

alias build_esp32="$Clear $ESP32_environment_variables $Cargo_esp build $ESP32_target"
alias run_esp32="$Clear $ESP32_environment_variables $Cargo_esp run $ESP32_target"
alias test_esp32="$Clear $ESP32_environment_variables $Cargo_esp test $ESP32_target"
alias check_esp32="$Clear $ESP32_environment_variables $Cargo_esp clippy $ESP32_target"

alias build_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_esp build $ESP32_S3_target"
alias run_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_esp run $ESP32_S3_target"
alias test_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_esp test $ESP32_S3_target"
alias check_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_esp clippy $ESP32_S3_target"

alias clean="$Cargo clean"

alias doc="$Linux_environment_variables $Cargo doc"

alias format="$Cargo fmt --all"

export RUST_BACKTRACE=1

alias update_export="source Export.sh"