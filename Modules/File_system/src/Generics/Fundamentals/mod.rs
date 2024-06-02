use std::{ops, u64};

pub mod Path;
pub use Path::*;
use Shared::Discriminant_trait;

#[derive(Default, PartialOrd, PartialEq, Eq, Ord, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Size_type(pub u64);
pub type Signed_size_type = i64;

pub enum Position_type {
    Start(Size_type),
    Current(Signed_size_type),
    End(Signed_size_type),
}

impl Position_type {
    pub fn From(Discriminant: u32, Value: u64) -> Self {
        match Discriminant {
            0 => Position_type::Start(Size_type(Value)),
            1 => Position_type::Current(Value as i64),
            2 => Position_type::End(Value as i64),
            _ => panic!("Invalid discriminant"),
        }
    }
}

impl Discriminant_trait for Position_type {
    fn Get_discriminant(&self) -> u32 {
        match self {
            Position_type::Start(_) => 0,
            Position_type::Current(_) => 1,
            Position_type::End(_) => 2,
        }
    }

    fn From_discriminant(Discriminant: u32) -> Self {
        match Discriminant {
            0 => Position_type::Start(Size_type::default()),
            1 => Position_type::Current(0),
            2 => Position_type::End(0),
            _ => panic!("Invalid discriminant"),
        }
    }
}

#[repr(transparent)]
pub struct Block_type(pub [u8; 512]);

impl Default for Block_type {
    fn default() -> Self {
        Block_type([0; 512])
    }
}

impl ops::Add<Size_type> for Size_type {
    type Output = Size_type;

    fn add(self, rhs: Size_type) -> Self::Output {
        Size_type(self.0 + rhs.0)
    }
}

impl From<u64> for Size_type {
    fn from(item: u64) -> Self {
        Size_type(item)
    }
}

impl From<usize> for Size_type {
    fn from(item: usize) -> Self {
        Size_type(item as u64)
    }
}
