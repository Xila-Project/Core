use core::time::Duration;
use file_system::{DeviceTrait, Error, Result, Size};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// JavaScript Date.now() - returns milliseconds since UNIX epoch
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}

/// WASM Time Device
///
/// Provides access to current time information in WASM environments.
/// Uses JavaScript's Date.now() API to get the current time since UNIX epoch.
///
/// The device returns time as a `Duration` struct containing seconds and nanoseconds
/// since UNIX epoch, similar to the native implementation.
pub struct TimeDevice;

impl Default for TimeDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeDevice {
    pub fn new() -> Self {
        Self {}
    }

    /// Get current time as Duration since UNIX epoch
    ///
    /// Uses JavaScript's Date.now() which returns milliseconds since UNIX epoch.
    /// Converts to seconds and nanoseconds for compatibility with Duration.
    fn get_current_time() -> Result<Duration> {
        // Get milliseconds since UNIX epoch from JavaScript Date.now()
        let millis = date_now();

        // Validate that we got a valid timestamp
        if !millis.is_finite() || millis < 0.0 {
            return Err(Error::InternalError);
        }

        // Convert to seconds and nanoseconds
        let secs = (millis / 1000.0) as u64;
        let nanos = ((millis % 1000.0) * 1_000_000.0) as u32;

        Ok(Duration::new(secs, nanos))
    }
}

impl DeviceTrait for TimeDevice {
    fn read(&self, buffer: &mut [u8]) -> Result<Size> {
        let duration = Self::get_current_time()?;

        let duration_bytes = unsafe {
            core::slice::from_raw_parts(
                &duration as *const Duration as *const u8,
                core::mem::size_of::<Duration>(),
            )
        };

        if buffer.len() < duration_bytes.len() {
            return Err(Error::InvalidParameter);
        }

        buffer[..duration_bytes.len()].copy_from_slice(duration_bytes);

        Ok(duration_bytes.len().into())
    }

    fn write(&self, _: &[u8]) -> Result<Size> {
        Err(Error::UnsupportedOperation)
    }

    fn get_size(&self) -> Result<Size> {
        Ok(core::mem::size_of::<Duration>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> Result<Size> {
        Err(Error::UnsupportedOperation)
    }

    fn flush(&self) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_device_creation() {
        let device = TimeDevice::new();
        assert_eq!(
            device.get_size().unwrap().as_u64(),
            core::mem::size_of::<Duration>() as u64
        );
    }

    #[test]
    fn test_time_device_read() {
        let device = TimeDevice::new();
        let mut buffer = [0u8; core::mem::size_of::<Duration>()];

        let result = device.read(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().as_u64(),
            core::mem::size_of::<Duration>() as u64
        );
    }

    #[test]
    fn test_time_device_write_unsupported() {
        let device = TimeDevice::new();
        let buffer = [0u8; 8];

        let result = device.write(&buffer);
        assert!(matches!(result, Err(Error::UnsupportedOperation)));
    }

    #[test]
    fn test_time_device_set_position_unsupported() {
        let device = TimeDevice::new();
        let position = file_system::Position::Start(0);

        let result = device.set_position(&position);
        assert!(matches!(result, Err(Error::UnsupportedOperation)));
    }

    #[test]
    fn test_time_device_flush_unsupported() {
        let device = TimeDevice::new();

        let result = device.flush();
        assert!(matches!(result, Err(Error::UnsupportedOperation)));
    }
}
