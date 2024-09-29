use std::ops;

mod Directory;
mod Flags;
mod Identifiers;
mod Path;
mod Permission;
mod Statistics;
mod Size;
mod Position;

pub use Directory::*;
pub use Flags::*;
pub use Identifiers::*;
pub use Path::*;
pub use Permission::*;
pub use Statistics::*;
pub use Size::*;
pub use Position::*;

#[repr(transparent)]
pub struct Block_type(pub [u8; 512]);

impl Default for Block_type {
    fn default() -> Self {
        Block_type([0; 512])
    }
}


