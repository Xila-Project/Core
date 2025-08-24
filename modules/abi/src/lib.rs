#![no_std]

extern crate alloc;

mod abort;
mod array;
mod assert;
mod context;
mod file_system;
mod mathematics;
mod memory;
mod string;
mod task;
mod time;
mod user;

pub use abort::*;
pub use array::*;
pub use assert::*;
pub use context::*;
pub use file_system::*;
pub use mathematics::*;
pub use memory::*;
pub use string::*;
pub use task::*;
pub use time::*;
pub use user::*;

#[cfg(test)]
mod tests {
    use drivers::standard_library::memory::MemoryManager;
    use memory::instantiate_global_allocator;

    instantiate_global_allocator!(MemoryManager);
}
