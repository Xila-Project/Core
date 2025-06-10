//#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate alloc;

mod Context;
mod File_system;
mod Memory;
mod Task;
mod Time;
mod User;

pub use Context::*;
pub use File_system::*;
pub use Memory::*;
pub use Task::*;
pub use Time::*;
pub use User::*;

#[cfg(test)]
mod Tests {
    use Drivers::Std::Memory::Memory_manager_type;
    use Memory::Instantiate_global_allocator;

    Instantiate_global_allocator!(Memory_manager_type);
}
