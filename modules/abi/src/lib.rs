#![no_std]

extern crate alloc;

mod context;
mod file_system;
mod memory;
mod string;
mod task;
mod time;
mod user;

pub use context::*;
pub use file_system::*;
pub use memory::*;
pub use string::*;
pub use task::*;
pub use time::*;
pub use user::*;

#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    drivers::standard_library::memory::instantiate_global_allocator!();
}
