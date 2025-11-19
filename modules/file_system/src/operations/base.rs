//! Device abstraction for storage and I/O operations.
//!
//! This module provides the core device trait and types for abstracting various
//! storage devices, peripherals, and I/O endpoints in the file system.

use crate::{Context, ControlArgument, ControlCommand, Error, Position, Result, Size};

/// Core trait for all device implementations in the file system.
///
/// A device represents any storage medium or I/O endpoint that can be read from and written to.
/// This includes physical storage devices (hard drives, SSDs, SD cards), memory devices for testing,
/// partition devices, and other specialized I/O devices.
///
/// ## Thread Safety
///
/// All device implementations must be thread-safe (`Send + Sync`) as they may be accessed
/// by multiple tasks/threads concurrently. Implementations should use appropriate synchronization
/// primitives like `RwLock` or `Mutex` to handle concurrent access.
///
/// ## Non-Blocking Operations
///
/// Devices should never block indefinitely. If an operation would block, implementations should
/// return [`Error::RessourceBusy`] instead. This means implementations should prefer
/// `try_read()` and `try_write()` variants of synchronization primitives.
///
/// ## Position Management
///
/// Devices maintain an internal position cursor that affects read and write operations.
/// The position can be manipulated using [`DeviceTrait::set_position`].
///
/// # Examples
///
/// ```rust
/// # extern crate alloc;
/// # use file_system::{Size, MemoryDevice, DirectBaseOperations, Position};
///
/// // Create a memory device for testing
/// let device = MemoryDevice::<512>::new(1024);
///
/// // Write data
/// let data = b"Hello, World!";
/// let bytes_written = device.write(data, 0).unwrap();
/// assert_eq!(bytes_written, data.len());
///
/// // Reset position and read back
/// device.set_position(0, &Position::Start(0)).unwrap();
/// let mut buffer = alloc::vec![0u8; data.len()];
/// let bytes_read = device.read(&mut buffer, 0).unwrap();
/// assert_eq!(bytes_read, data.len());
/// assert_eq!(&buffer, data);
/// ```
pub trait BaseOperations: Send + Sync {
    fn open(&self, _context: &mut Context) -> Result<()> {
        Ok(())
    }

    fn close(&self, _context: &mut Context) -> Result<()> {
        Ok(())
    }

    /// Read data from the device at the current position.
    ///
    /// Reads up to `Buffer.len()` bytes from the device into the provided buffer.
    /// The actual number of bytes read may be less than requested.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    /// * `Buffer` - Mutable byte slice to read data into
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - Number of bytes successfully read
    /// * `Err(Error)` - Error if read operation failed
    ///
    /// # Errors
    ///
    /// * [`Error::InputOutput`] - I/O error during read operation
    /// * [`Error::RessourceBusy`] - Device is temporarily unavailable
    /// * [`Error::InvalidParameter`] - Invalid buffer or device state
    fn read(
        &self,
        context: &mut Context,
        buffer: &mut [u8],
        absolute_position: Size,
    ) -> Result<usize>;

    fn read_until(
        &self,
        context: &mut Context,
        buffer: &mut [u8],
        absolute_position: Size,
        delimiter: &[u8],
    ) -> Result<usize> {
        if delimiter.is_empty() {
            return Err(Error::InvalidParameter);
        }

        let mut total_read = 0;
        let mut match_count = 0;

        while total_read < buffer.len() {
            let bytes_read = self.read(
                context,
                &mut buffer[total_read..total_read + 1],
                absolute_position + total_read as Size,
            )?;

            if bytes_read == 0 {
                break; // End of file or no more data
            }

            total_read += bytes_read;

            // Check if we're matching the delimiter
            if buffer[total_read - 1] == delimiter[match_count] {
                match_count += 1;
                if match_count == delimiter.len() {
                    break; // Found complete delimiter
                }
            } else {
                // Reset and check if current byte starts a new match
                match_count = 0;
                if buffer[total_read - 1] == delimiter[0] {
                    match_count = 1;
                }
            }
        }

        Ok(total_read)
    }

    /// Write data to the device at the current position.
    ///
    /// Writes up to `Buffer.len()` bytes from the buffer to the device.
    /// The actual number of bytes written may be less than requested.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    /// * `Buffer` - Byte slice containing data to write
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - Number of bytes successfully written
    /// * `Err(Error)` - Error if write operation failed
    ///
    /// # Errors
    ///
    /// * [`Error::InputOutput`] - I/O error during write operation
    /// * [`Error::NoSpaceLeft`] - Device is full
    /// * [`Error::RessourceBusy`] - Device is temporarily unavailable
    /// * [`Error::PermissionDenied`] - Device is read-only
    fn write(&self, context: &mut Context, buffer: &[u8], absolute_position: Size)
    -> Result<usize>;

    fn write_pattern(
        &self,
        context: &mut Context,
        pattern: &[u8],
        count: usize,
        absolute_position: Size,
    ) -> Result<usize> {
        let mut total_written = 0;

        for _ in 0..count {
            let bytes_written =
                self.write(context, pattern, absolute_position + total_written as Size)?;
            if bytes_written == 0 {
                break; // Unable to write more
            }
            total_written += bytes_written;
        }

        Ok(total_written)
    }

    fn write_vectored(
        &self,
        context: &mut Context,
        buffers: &[&[u8]],
        absolute_position: Size,
    ) -> Result<usize> {
        let mut total_written = 0;

        for buffer in buffers {
            if buffer.is_empty() {
                continue; // Skip empty buffers
            }
            let bytes_written =
                self.write(context, buffer, absolute_position + total_written as Size)?;
            if bytes_written == 0 {
                break; // Unable to write more
            }
            total_written += bytes_written;
        }

        Ok(total_written)
    }

    /// Set the current position cursor for read/write operations.
    ///
    /// The position affects where subsequent read and write operations will occur.
    /// Different position types allow for absolute positioning, relative positioning,
    /// and positioning from the end of the device.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    /// * `Position` - The new position to set
    ///
    /// # Returns
    ///
    /// * `Ok(Size)` - The new absolute position after the operation
    /// * `Err(Error)` - Error if position is invalid
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidParameter`] - Position is beyond device bounds
    fn set_position(
        &self,
        _context: &mut Context,
        _current_position: Size,
        _position: &Position,
    ) -> Result<Size> {
        match _position {
            Position::Start(position) => Ok(*position),
            Position::Current(offset) => Ok(_current_position.wrapping_add(*offset as Size)),
            Position::End(_) => Err(Error::UnsupportedOperation),
        }
    }

    /// Flush any buffered data to the underlying storage.
    ///
    /// Ensures that all pending write operations are committed to the physical device.
    /// This is important for data integrity, especially on buffered devices.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Flush completed successfully
    /// * `Err(Error)` - Error during flush operation
    fn flush(&self, _context: &mut Context) -> Result<()> {
        Ok(())
    }

    fn control(
        &self,
        _context: &mut Context,
        _command: ControlCommand,
        _argument: &mut ControlArgument,
    ) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn clone_context(&self, context: &Context) -> Result<Context>;
}

pub fn open_close_operation<D, R>(device: &D, operation: impl Fn(&D) -> Result<R>) -> Result<R>
where
    D: DirectBaseOperations,
{
    device.open()?;
    let result = operation(device);
    device.close()?;
    result
}

pub trait DirectBaseOperations: Send + Sync {
    fn open(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }

    fn read(&self, buffer: &mut [u8], absolute_position: Size) -> Result<usize>;

    fn read_until(
        &self,
        buffer: &mut [u8],
        absolute_position: Size,
        delimiter: &[u8],
    ) -> Result<usize> {
        if delimiter.is_empty() {
            return Err(Error::InvalidParameter);
        }

        let mut total_read = 0;
        let mut match_count = 0;

        while total_read < buffer.len() {
            let bytes_read = self.read(
                &mut buffer[total_read..total_read + 1],
                absolute_position + total_read as Size,
            )?;

            if bytes_read == 0 {
                break; // End of file or no more data
            }

            total_read += bytes_read;

            // Check if we're matching the delimiter
            if buffer[total_read - 1] == delimiter[match_count] {
                match_count += 1;
                if match_count == delimiter.len() {
                    break; // Found complete delimiter
                }
            } else {
                // Reset and check if current byte starts a new match
                match_count = 0;
                if buffer[total_read - 1] == delimiter[0] {
                    match_count = 1;
                }
            }
        }

        Ok(total_read)
    }

    fn write(&self, buffer: &[u8], absolute_position: Size) -> Result<usize>;

    fn write_pattern(
        &self,
        pattern: &[u8],
        count: usize,
        absolute_position: Size,
    ) -> Result<usize> {
        let mut total_written = 0;

        for _ in 0..count {
            let bytes_written = self.write(pattern, absolute_position + total_written as Size)?;
            if bytes_written == 0 {
                break; // Unable to write more
            }
            total_written += bytes_written;
        }

        Ok(total_written)
    }

    fn write_vectored(&self, buffers: &[&[u8]], absolute_position: Size) -> Result<usize> {
        let mut total_written = 0;

        for buffer in buffers {
            if buffer.is_empty() {
                continue; // Skip empty buffers
            }
            let bytes_written = self.write(buffer, absolute_position + total_written as Size)?;
            if bytes_written == 0 {
                break; // Unable to write more
            }
            total_written += bytes_written;
        }

        Ok(total_written)
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }

    fn set_position(&self, _current_position: Size, _position: &Position) -> Result<Size> {
        match _position {
            Position::Start(position) => Ok(*position),
            Position::Current(offset) => Ok(_current_position.wrapping_add(*offset as Size)),
            Position::End(_) => Err(Error::UnsupportedOperation),
        }
    }

    fn control(&self, _command: ControlCommand, _argument: &mut ControlArgument) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }
}

impl<T> BaseOperations for T
where
    T: DirectBaseOperations + Send + Sync,
{
    fn open(&self, _: &mut Context) -> Result<()> {
        self.open()
    }

    fn read(&self, _: &mut Context, buffer: &mut [u8], absolute_position: Size) -> Result<usize> {
        self.read(buffer, absolute_position)
    }

    fn write(&self, _: &mut Context, buffer: &[u8], absolute_position: Size) -> Result<usize> {
        self.write(buffer, absolute_position)
    }

    fn write_pattern(
        &self,
        _: &mut Context,
        pattern: &[u8],
        count: usize,
        absolute_position: Size,
    ) -> Result<usize> {
        self.write_pattern(pattern, count, absolute_position)
    }

    fn write_vectored(
        &self,
        _context: &mut Context,
        buffers: &[&[u8]],
        absolute_position: Size,
    ) -> Result<usize> {
        self.write_vectored(buffers, absolute_position)
    }

    fn flush(&self, _: &mut Context) -> Result<()> {
        self.flush()
    }

    fn set_position(
        &self,
        _context: &mut Context,
        current_position: Size,
        position: &Position,
    ) -> Result<Size> {
        self.set_position(current_position, position)
    }

    fn control(
        &self,
        _: &mut Context,
        command: ControlCommand,
        argument: &mut ControlArgument,
    ) -> Result<()> {
        self.control(command, argument)
    }

    fn clone_context(&self, _context: &Context) -> Result<Context> {
        Ok(Context::new_empty())
    }
}
