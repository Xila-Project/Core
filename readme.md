<h1 align="center">Xila core</h1>

## 📝 Description

This repository contains the core of Xila. It contains the code for xila's core functionalities like task management, file system, virtual machine, graphics, drivers, etc.

## 🛠️ Build

### ✅ Requirements

Please install the following dependencies to build core:

- `gcc-multilib`: for cross-compilation purposes.
- [`cargo-make`](https://github.com/sagiegurari/cargo-make): to build the project with `cargo`.
- `nodejs`: to generate fonts for LVGL ([lv_font_conv](https://github.com/lvgl/lv_font_conv)).
- `wasm32-wasip1` Rust target: to compile xila virtual machine executables.
- (Optional) `wasm32-unknown-unknown` Rust target: to compile `wasm_example`.
- (Optional) `nightly` Rust toolchain: to compile the `wasm_example`.
- (Optional) [`trunk`](https://trunkrs.dev/): to build the `wasm_example`.
- (Optional) Rust [xtensa-esp32\*-espidf](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html) toolchain: to compile for ESP32 / ESP32-S series.
- (Optional) Rust [riscv\*-esp-espidf](https://docs.esp-rs.org/book/installation/riscv.html): to compile for ESP32-H / ESP32-C series.

### 🛠️ Instructions

1. Clone the repository:

```bash
git clone https://github.com/Xila-Project/Core.git
```

2. Navigate to the project directory:

```bash
cd Core
```

3. Run one of the examples:

```bash
cargo make run -p native_example
```

or

```bash
cargo make generate-fonts
cd examples/wasm && trunk serve
```

## ℹ️ About

This project is under the [MIT license](License.md).
