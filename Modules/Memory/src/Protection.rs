//! Memory protection flags and operations.
//!
//! This module provides functionality to represent and manipulate memory protection
//! settings (read, write, execute) and interfaces for applying these protections.

use core::fmt::Debug;

/// Represents memory protection flags.
///
/// This structure encapsulates read, write, and execute permissions for memory regions
/// in a compact bit representation.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Protection_type(u8);

impl Debug for Protection_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Formatter
            .debug_struct("Protection_type")
            .field("Read", &self.Get_read())
            .field("Write", &self.Get_write())
            .field("Execute", &self.Get_execute())
            .finish()
    }
}

impl Protection_type {
    /// Bit flag representing read permission.
    pub const Read_bit: u8 = 1 << 0;

    /// Bit flag representing write permission.
    pub const Write_bit: u8 = 1 << 1;

    /// Bit flag representing execute permission.
    pub const Execute_bit: u8 = 1 << 2;

    /// No memory access permissions.
    pub const None: Self = Self(0);

    /// Read-only memory access.
    pub const Read: Self = Self(Self::Read_bit);

    /// Write-only memory access.
    pub const Write: Self = Self(Self::Write_bit);

    /// Execute-only memory access.
    pub const Execute: Self = Self(Self::Execute_bit);

    /// Read and write memory access.
    pub const Read_write: Self = Self(Self::Read_bit | Self::Write_bit);

    /// Read and execute memory access.
    pub const Read_execute: Self = Self(Self::Read_bit | Self::Execute_bit);

    /// Write and execute memory access.
    pub const Write_execute: Self = Self(Self::Write_bit | Self::Execute_bit);

    /// Full memory access (read, write, and execute).
    pub const Read_write_execute: Self = Self(Self::Read_bit | Self::Write_bit | Self::Execute_bit);

    /// Creates a new protection type with specified permissions.
    ///
    /// # Parameters
    /// - `Read`: Whether read permission is granted
    /// - `Write`: Whether write permission is granted
    /// - `Execute`: Whether execute permission is granted
    ///
    /// # Returns
    /// A new protection type with the specified permissions.
    pub const fn New(Read: bool, Write: bool, Execute: bool) -> Self {
        Self(0).Set_read(Read).Set_write(Write).Set_execute(Execute)
    }

    /// Sets or clears specified bits in the protection flags.
    ///
    /// # Parameters
    /// - `Bits`: The bits to modify
    /// - `Value`: Whether to set or clear the bits
    ///
    /// # Returns
    /// A new protection type with the modified bits.
    const fn Set_bits(mut self, Bits: u8, Value: bool) -> Self {
        if Value {
            self.0 |= Bits;
        } else {
            self.0 &= !Bits;
        }
        self
    }

    /// Checks if any of the specified bits are set.
    ///
    /// # Parameters
    /// - `Bits`: The bits to check
    ///
    /// # Returns
    /// `true` if any of the specified bits are set, `false` otherwise.
    const fn Get_bits(&self, Bits: u8) -> bool {
        self.0 & Bits != 0
    }

    /// Sets or clears the read permission.
    ///
    /// # Parameters
    /// - `Value`: Whether to grant or revoke read permission
    ///
    /// # Returns
    /// A new protection type with the modified read permission.
    pub const fn Set_read(self, Value: bool) -> Self {
        self.Set_bits(Self::Read_bit, Value)
    }

    /// Sets or clears the write permission.
    ///
    /// # Parameters
    /// - `Value`: Whether to grant or revoke write permission
    ///
    /// # Returns
    /// A new protection type with the modified write permission.
    pub const fn Set_write(self, Value: bool) -> Self {
        self.Set_bits(Self::Write_bit, Value)
    }

    /// Sets or clears the execute permission.
    ///
    /// # Parameters
    /// - `Value`: Whether to grant or revoke execute permission
    ///
    /// # Returns
    /// A new protection type with the modified execute permission.
    pub const fn Set_execute(self, Value: bool) -> Self {
        self.Set_bits(Self::Execute_bit, Value)
    }

    /// Checks if read permission is granted.
    ///
    /// # Returns
    /// `true` if read permission is granted, `false` otherwise.
    pub const fn Get_read(&self) -> bool {
        self.Get_bits(Self::Read_bit)
    }

    /// Checks if write permission is granted.
    ///
    /// # Returns
    /// `true` if write permission is granted, `false` otherwise.
    pub const fn Get_write(&self) -> bool {
        self.Get_bits(Self::Write_bit)
    }

    /// Checks if execute permission is granted.
    ///
    /// # Returns
    /// `true` if execute permission is granted, `false` otherwise.
    pub const fn Get_execute(&self) -> bool {
        self.Get_bits(Self::Execute_bit)
    }

    /// Converts the protection type to its raw u8 representation.
    ///
    /// # Returns
    /// The raw byte value representing the protection flags.
    pub const fn As_u8(&self) -> u8 {
        self.0
    }

    /// Creates a new protection type from a raw u8 value.
    ///
    /// # Parameters
    /// - `Value`: The raw protection flags
    ///
    /// # Returns
    /// A new protection type with the specified flags.
    pub const fn From_u8(Value: u8) -> Self {
        Self(Value)
    }
}

impl From<Protection_type> for u8 {
    fn from(Protection: Protection_type) -> Self {
        Protection.0
    }
}

impl From<u8> for Protection_type {
    fn from(Protection: u8) -> Self {
        Self(Protection)
    }
}

/// Interface for implementing memory protection operations.
///
/// This trait should be implemented by memory managers or other components
/// that can apply protection settings to memory regions.
pub trait Protection_trait {}
