use std::sync::RwLock;

use esp_idf_sys::{self, gpio_reset_pin};
use File_system::Device_trait;

use super::{Error_type, Result_type};

#[macro_export]
macro_rules! Enumerate_pin_devices {
    (
        $($Pin:literal),*) => {
        unsafe {
            [
                $((
                    $Pin,
                    Path_type::New_unchecked_constant(concat!("/Devices/Pin", $Pin))
                ),)*
            ] }
    };
}

pub fn Mount_pin_devices(
    Virtual_file_system: &File_system::Virtual_file_system_type,
    Pin_devices: &'static [(u8, &'static File_system::Path_type)],
) -> Result_type<()> {
    for (i, Path) in Pin_devices.iter() {
        Virtual_file_system
            .Add_device(Path, Box::new(Pin_device_type::New(*i)))
            .map_err(|_| Error_type::Failed_to_register_pin_device)?;
    }

    Ok(())
}

struct Inner_type {
    Pin: u8,
}

#[repr(transparent)]
pub struct Pin_device_type(RwLock<Inner_type>);

impl Pin_device_type {
    pub fn New(Pin: u8) -> Self {
        unsafe {
            gpio_reset_pin(Pin as i32);
        }

        Pin_device_type(RwLock::new(Inner_type { Pin }))
    }
}

impl Device_trait for Pin_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<usize> {
        if Buffer.len() != 2 {
            return Err(File_system::Error_type::Invalid_input);
        }

        let Inner = self.0.read()?;

        match Buffer[0] {
            1 => {
                Buffer[1] = unsafe { esp_idf_sys::gpio_get_level(Inner.Pin as i32) as u8 };
            }
            _ => {
                return Err(File_system::Error_type::Invalid_input);
            }
        }

        Ok(2)
    }

    fn Write(&self, Buffer: &[u8]) -> File_system::Result_type<usize> {
        if Buffer.len() != 2 {
            return Err(File_system::Error_type::Invalid_input);
        }

        let Inner = self.0.write()?;

        match Buffer[0] {
            0 => unsafe {
                esp_idf_sys::gpio_set_direction(Inner.Pin as i32, Buffer[1] as u32);
            },
            1 => unsafe {
                esp_idf_sys::gpio_set_level(Inner.Pin as i32, Buffer[1] as u32);
            },
            2 => unsafe {
                esp_idf_sys::gpio_set_pull_mode(Inner.Pin as i32, Buffer[1] as u32);
            },
            _ => {
                return Err(File_system::Error_type::Invalid_input);
            }
        }

        Ok(2)
    }

    fn Get_size(&self) -> File_system::Result_type<usize> {
        Ok(2)
    }

    fn Set_position(&self, _: &File_system::Position_type) -> File_system::Result_type<usize> {
        Ok(0)
    }

    fn Flush(&self) -> File_system::Result_type<()> {
        Ok(())
    }
}
