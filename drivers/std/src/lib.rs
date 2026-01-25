#![cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]

pub mod console;
pub mod devices;
pub mod drive_file;
pub mod executor;
pub mod io;
pub mod loader;
pub mod log;
pub mod memory;
pub mod tuntap;

pub extern crate memory as memory_exported;

#[cfg(test)]
mod tests {
    extern crate abi_definitions;
}
