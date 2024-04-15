#!/bin/bash
# Execute this file with: `source Export.sh`

clear && echo "Setting up environment variables for Rust cross-compilation"

# Variables

Linux_environment_variables="DEP_LV_CONFIG_PATH=\"$PWD/Modules/LVGL\""
Linux_target="--target x86_64-unknown-linux-gnu"
Windows_target="--target x86_64-pc-windows-gnu"

ESP32_environment_variables="MCU=esp32 DEP_LV_CONFIG_PATH=\"$PWD/Modules/LVGL\""
ESP32_target="--target xtensa-esp32-espidf --features ESP32 -Z build-std=std,panic_abort"
ESP32_S3_environment_variables="MCU=esp32s3 DEP_LV_CONFIG_PATH=\"$PWD/Modules/LVGL\""
ESP32_S3_target="--target xtensa-esp32s3-espidf --features ESP32_S3 -Z build-std=std,panic_abort"

Cargo="cargo"

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

# Aliases
Clear="clear &&"
Cargo_build="$Cargo build"
alias build_linux="$Clear $Linux_environment_variables $Cargo_build $Linux_target"
alias build_windows="$Clear $Cargo_build $Windows_target"
alias build_esp32="$Clear $ESP32_environment_variables $Cargo_build $ESP32_target"
alias build_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_build $ESP32_S3_target"

Cargo_run="$Cargo run"
alias run_linux="$Clear $Linux_environment_variables $Cargo_run $Linux_target"
alias run_windows="$Clear $Cargo_run $Windows_target"
alias run_esp32="$Clear $ESP32_environment_variables $Cargo_run $ESP32_target"
alias run_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_run $ESP32_S3_target"

Cargo_test="RUST_MIN_STACK=8388608 $Cargo test"
alias test_linux="$Clear $Linux_environment_variables $Cargo_test $Linux_target"
alias test_windows="$Clear $Cargo_test $Windows_target"
alias test_esp32="$Clear $ESP32_environment_variables $Cargo_test $ESP32_target"
alias test_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_test $ESP32_S3_target"

Cargo_check="$Cargo clippy"
alias check_linux="$Clear $Linux_environment_variables $Cargo_check $Linux_target"
alias check_windows="$Clear $Cargo_check $Windows_target"
alias check_esp32="$Clear $ESP32_environment_variables $Cargo_check $ESP32_target"
alias check_esp32_s3="$Clear $ESP32_S3_environment_variables $Cargo_check $ESP32_S3_target"

alias clean="$Cargo clean"

alias doc="$Linux_environment_variables $Cargo doc"

alias format="$Cargo fmt"

export RUST_BACKTRACE=1

alias update_export="source Export.sh"