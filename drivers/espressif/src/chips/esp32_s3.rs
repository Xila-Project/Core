use file_system::{Path, VirtualFileSystem};

use crate::Enumerate_pin_devices;

use crate::Espressif::Shared::{self, Result};

// The ESP32-S3 chip features 45 physical GPIO pins:
// - GPIO0 ~ GPIO21
// - GPIO26 ~ GPIO48
const Pin_devices: [(u8, &Path); 45] = Enumerate_pin_devices!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 26, 27, 28, 29,
    30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48
);

pub fn Mount_pin_devices(Virtual_file_system: &VirtualFileSystem) -> Result<()> {
    Shared::Mount_pin_devices(Virtual_file_system, &Pin_devices)
}
