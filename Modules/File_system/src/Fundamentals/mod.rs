mod Entry;
mod Flags;
mod Identifiers;
mod Metadata;
mod Path;
mod Permission;
mod Position;
mod Size;
mod Statistics;

pub use Entry::*;
pub use Flags::*;
pub use Identifiers::*;
pub use Metadata::*;
pub use Path::*;
pub use Permission::*;
pub use Position::*;
pub use Size::*;
pub use Statistics::*;

#[repr(transparent)]
pub struct Block_type(pub [u8; 512]);

impl Default for Block_type {
    fn default() -> Self {
        Block_type([0; 512])
    }
}
