use file_system::{Path_type, Virtual_file_system_type};

use crate::Enumerate_pin_devices;

use crate::Espressif::Shared::{self, Result_type};

/// The ESP32 chip features 34 physical GPIO pins :
/// - GPIO0 ~ GPIO19
/// - GPIO21 ~ GPIO23
/// - GPIO25 ~ GPIO27
/// - GPIO32 ~ GPIO39
const Pin_devices: [(u8, &Path_type); 34] = Enumerate_pin_devices!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 21, 22, 23, 25, 26, 27,
    32, 33, 34, 35, 36, 37, 38, 39
);

pub fn Mount_pin_devices(
    Virtual_file_system: &'static Virtual_file_system_type,
) -> Result_type<()> {
    Shared::Mount_pin_devices(Virtual_file_system, &Pin_devices)
}
