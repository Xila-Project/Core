#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

mod context;
mod file_system;
mod memory;
mod task;
mod time;
mod user;

pub use context::*;
pub use file_system::*;
pub use memory::*;
pub use task::*;
pub use time::*;
pub use user::*;

#[cfg(test)]
mod tests {
    use drivers::Std::Memory::Memory_manager_type;
    use memory::Instantiate_global_allocator;

    Instantiate_global_allocator!(Memory_manager_type);
}
