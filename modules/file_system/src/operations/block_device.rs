use crate::{BaseOperations, DirectFileOperations, Result, Size};

pub trait BlockDevice: BaseOperations {
    /// Get the total size of the device in bytes.
    ///
    /// Returns the maximum amount of data that can be stored on or read from the device.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - Total device size in bytes
    /// * `Err(Error)` - Error if size cannot be determined
    fn get_size(&self, context: &mut crate::Context) -> Result<Size> {
        Ok(self.get_block_count(context)? * self.get_block_size(context)? as u64)
    }

    /// Get the block size of the device in bytes.
    ///
    /// For block devices, this returns the minimum unit of data transfer.
    /// Operations should ideally be aligned to block boundaries for optimal performance.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Block size in bytes
    /// * `Err(Error::UnsupportedOperation)` - Device doesn't have a block size
    fn get_block_size(&self, _context: &mut crate::Context) -> Result<usize>;

    fn get_block_count(&self, context: &mut crate::Context) -> Result<Size>;
}

pub trait DirectBlockDevice: DirectFileOperations {
    fn get_size(&self) -> Result<Size> {
        Ok(self.get_block_count()? * self.get_block_size()? as Size)
    }

    fn erase(&self, _absolute_position: Size) -> Result<()> {
        Err(crate::Error::UnsupportedOperation)
    }

    fn get_block_size(&self) -> Result<usize>;

    fn get_block_count(&self) -> Result<Size>;
}

impl<T> BlockDevice for T
where
    T: DirectBlockDevice,
{
    fn get_size(&self, _: &mut crate::Context) -> Result<Size> {
        self.get_size()
    }

    fn get_block_size(&self, _: &mut crate::Context) -> Result<usize> {
        self.get_block_size()
    }

    fn get_block_count(&self, _context: &mut crate::Context) -> Result<Size> {
        self.get_block_count()
    }
}
