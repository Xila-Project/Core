#!/bin/bash
# Execute this file with: `source Export.sh`

clear && echo "Setting up environment variables for Rust cross-compilation"

# Variables
Linux_target="--target x86_64-unknown-linux-gnu --features Linux"
Windows_target="--target x86_64-pc-windows-gnu --features Windows"
ESP32_target="--target xtensa-esp32-none-elf --features ESP32"
ESP32_S3_target="--target xtensa-esp32s3-none-elf --features ESP32_S3"

Cargo="clear && cargo"

# Remove all aliases
unalias build_linux 
unalias build_windows 
unalias build_esp32 
unalias build_esp32_s3 
unalias run_linux
unalias run_windows 
unalias run_esp32
unalias run_esp32_s3
unalias clean

# Aliases
Cargo_build="$Cargo build"
alias build_linux="$Cargo_build $Linux_target"
alias build_windows="$Cargo_build build $Windows_target"
alias build_esp32="$Cargo_build build $ESP32_target"
alias build_esp32_s3="$Cargo_build build $ESP32_S3_target"

Cargo_run="$Cargo run"
alias run_linux="$Cargo_run $Linux_target"
alias run_windows="$Cargo_run $Windows_target"
alias run_esp32="$Cargo_run $ESP32_target"
alias run_esp32_s3="$Cargo_run $ESP32_S3_target"

Cargo_test="$Cargo test"
alias test_linux="$Cargo_test $Linux_target"
alias test_windows="$Cargo_test $Windows_target"
alias test_esp32="$Cargo_test $ESP32_target"
alias test_esp32_s3="$Cargo_test $ESP32_S3_target"

Cargo_check="$Cargo check"
alias check_linux="$Cargo_check $Linux_target"
alias check_windows="$Cargo_check $Windows_target"
alias check_esp32="$Cargo_check $ESP32_target"
alias check_esp32_s3="$Cargo_check $ESP32_S3_target"

alias clean="$Cargo clean"

export RUST_BACKTRACE=1

alias update_export="source Export.sh"