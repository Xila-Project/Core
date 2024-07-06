<h1 align="center">Xila core</h1>

## 📝 Description

This repository contains the core of Xila. It contains the code for xila's core functionalities like task management, file system, virtual machine, graphics, drivers, etc.

## 🛠️ Build

### ✅ Requirements

Please install the following dependencies to build core:

- `gcc-multilib` : to compile 32-bit applications on 64-bit systems with `gcc`.
- (Optional) Rust [xtensa-esp32*-espidf](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html) toolchain : to compile for ESP32 / ESP32-S series.
- (Optional) Rust [riscv*-esp-espidf](https://docs.esp-rs.org/book/installation/riscv.html) : to compile for ESP32-H / ESP32-C series.

### 🛠️ Instructions

1. Clone the repository:

```bash
git clone https://github.com/Xila-Project/Core.git
```

2. Change directory:

```bash
cd Core
```

3. Source `Export.sh`:

```bash
source Export.sh
```

4. Build for the corresponding target:

```bash
build_<target>
```

Currently supported targets are:
- `linux`
- `esp32`
- `esp32_s3`

## ℹ️ About

This project is under the [MIT license](License.md).