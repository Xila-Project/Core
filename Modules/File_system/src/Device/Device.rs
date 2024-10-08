use core::fmt;
use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use crate::{Error_type, Position_type, Result_type, Size_type};

#[macro_export]
macro_rules! Create_device {
    ($device:expr) => {
        $crate::Device_type::New(std::sync::Arc::new($device))
    };
}

/// A device is a file-like object that can be read from and written to.
///
/// This trait is used to abstract the underlying peripheral or file system.
/// A device should be thread-safe, as it may be accessed by multiple tasks/threads concurrently.
/// A device should never block and should return a [`Error_type::Ressource_busy`] error if the operation would block.
/// That means that the device should use use [`std::sync::RwLock::try_read`] and [`std::sync::RwLock::try_read`].
pub trait Device_trait: Send + Sync {
    /// Read data from the device.
    fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type>;

    /// Write data to the device.
    fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type>;

    /// Get the size of maximum data that can be read or written.
    fn Get_size(&self) -> Result_type<Size_type>;

    /// Set the position of the device.
    fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type>;

    /// Flush the device (write any buffered data).
    fn Flush(&self) -> Result_type<()>;

    /// Erase the device.
    ///
    /// This operation is only required for block devices.
    fn Erase(&self) -> Result_type<()> {
        Err(Error_type::Unsupported_operation)
    }

    /// Get the device block size.
    fn Get_block_size(&self) -> Result_type<usize> {
        Err(Error_type::Unsupported_operation)
    }

    fn Is_a_terminal(&self) -> bool {
        false
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Device_type(Arc<dyn Device_trait>);

impl Debug for Device_type {
    fn fmt(&self, Formatter: &mut Formatter) -> fmt::Result {
        write!(Formatter, "Device_type")
    }
}

impl Device_type {
    pub fn New(Device: Arc<dyn Device_trait>) -> Self {
        Device_type(Device)
    }

    pub fn Read(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        self.0.Read(Buffer)
    }

    pub fn Write(&self, Buffer: &[u8]) -> Result_type<Size_type> {
        self.0.Write(Buffer)
    }

    pub fn Get_size(&self) -> Result_type<Size_type> {
        self.0.Get_size()
    }

    pub fn Set_position(&self, Position: &Position_type) -> Result_type<Size_type> {
        self.0.Set_position(Position)
    }

    pub fn Flush(&self) -> Result_type<()> {
        self.0.Flush()
    }

    pub fn Erase(&self) -> Result_type<()> {
        self.0.Erase()
    }

    pub fn Get_block_size(&self) -> Result_type<usize> {
        self.0.Get_block_size()
    }

    pub fn Is_a_terminal(&self) -> bool {
        self.0.Is_a_terminal()
    }
}
