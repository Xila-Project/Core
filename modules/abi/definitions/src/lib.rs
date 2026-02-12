#![no_std]

extern crate alloc;

mod file_system;
mod memory;
mod network;
mod string;
mod task;
mod time;
mod user;

pub use file_system::*;
pub use memory::*;
pub use network::*;
pub use string::*;
pub use task::*;
pub use time::*;
pub use user::*;

#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    drivers_std::memory::instantiate_global_allocator!();
}
