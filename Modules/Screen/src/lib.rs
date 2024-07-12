#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

mod Area;
mod Color;
mod Error;
mod Point;

pub use Area::*;
pub use Color::*;
pub use Error::*;
pub use Point::*;

pub trait Screen_traits<T> {
    fn Update(&mut self, Area: Area_type, Buffer: &[T]) -> Result_type<()>;
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum Touch_type {
    Pressed,
    Released,
}

impl From<Touch_type> for u8 {
    fn from(Value: Touch_type) -> u8 {
        Value as u8
    }
}

impl TryFrom<u8> for Touch_type {
    type Error = ();

    fn try_from(Value: u8) -> Result<Self, Self::Error> {
        match Value {
            0 => Ok(Self::Pressed),
            1 => Ok(Self::Released),
            _ => Err(()),
        }
    }
}
