#![cfg(target_os = "espidf")]

// - Dependencies
// - - External dependencies
use embedded_hal::blocking::spi::*;

// - Traits
pub trait SPI_traits = Transfer<u8> + Write<u8>
where
    <Self as Transfer<u8>>::Error: core::fmt::Debug,
    <Self as Write<u8>>::Error: core::fmt::Debug;
pub trait Chip_select_traits = embedded_hal::digital::v2::OutputPin;

// - Modules
pub mod Drive;
pub use Drive::*;

pub mod Partition;
pub use Partition::*;

pub mod File_system;
pub use File_system::*;

pub mod File;
pub use File::*;
