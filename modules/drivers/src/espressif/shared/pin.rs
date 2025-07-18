use std::sync::RwLock;

use esp_idf_sys::{self, gpio_reset_pin};
use file_system::{DeviceTrait, Size};
use graphics::lvgl::input_device::Data;
use peripherals::{Direction_type, Pin_data_type, Pull_type};

use super::{Error, Result};

#[macro_export]
macro_rules! Enumerate_pin_devices {
    (
        $($Pin:literal),*) => {
        unsafe {
            [
                $((
                    $Pin,
                    Path_type::New_unchecked_constant(concat!("/devices/Pin", $Pin))
                ),)*
            ] }
    };
}

pub fn Mount_pin_devices(
    Virtual_file_system: &file_system::VirtualFileSystem,
    Pin_devices: &'static [(u8, &'static file_system::Path_type)],
) -> Result<()> {
    for (i, Path) in Pin_devices.iter() {
        Virtual_file_system
            .Add_device(Path, Box::new(Pin_device_type::new(*i)))
            .map_err(|_| Error::Failed_to_register_pin_device)?;
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

impl DeviceTrait for Pin_device_type {
    fn Read(&self, Buffer: &mut [u8]) -> file_system::Result<Size> {
        let Data: &mut Pin_data_type = Buffer
            .try_into()
            .map_err(|_| file_system::Error::Invalid_input)?;

        Data.Set_level(
            Level_type::try_from(unsafe {
                esp_idf_sys::gpio_get_level(self.0.read()?.Pin as i32) as u8
            })
            .map_err(|_| file_system::Error::Invalid_input)?,
        );

        Ok(size_of::<Pin_data_type>())
    }

    fn Write(&self, Buffer: &[u8]) -> file_system::Result<usize> {
        let Data: &mut Pin_data_type = Buffer
            .try_into()
            .map_err(|_| file_system::Error::Invalid_input)?;

        let Pin = self.0.read()? as i32;

        if let Some(Pin) = Data.get_direction() {
            match Pin {
                Direction_type::Input => unsafe {
                    esp_idf_sys::gpio_set_direction(
                        Pin,
                        esp_idf_sys::gpio_mode_t_GPIO_MODE_INPUT as u32,
                    );
                },
                Direction_type::Output => unsafe {
                    esp_idf_sys::gpio_set_direction(
                        Pin,
                        esp_idf_sys::gpio_mode_t_GPIO_MODE_OUTPUT as u32,
                    );
                },
            }
        }

        if let Some(Level) = Data.get_level() {
            unsafe {
                esp_idf_sys::gpio_set_level(Pin, Level.into() as u32);
            }
        }

        if let Some(Pull) = Data.get_pull() {
            match Pull {
                Pull_type::None => unsafe {
                    esp_idf_sys::gpio_set_pull_mode(
                        Pin,
                        esp_idf_sys::gpio_pull_mode_t_GPIO_FLOATING as u32,
                    );
                },
                Pull_type::Up => unsafe {
                    esp_idf_sys::gpio_set_pull_mode(
                        Pin,
                        esp_idf_sys::gpio_pull_mode_t_GPIO_PULLUP_ONLY as u32,
                    );
                },
                Pull_type::Down => unsafe {
                    esp_idf_sys::gpio_set_pull_mode(
                        Pin,
                        esp_idf_sys::gpio_pull_mode_t_GPIO_PULLDOWN_ONLY as u32,
                    );
                },
                Pull_type::Up_down => unsafe {
                    esp_idf_sys::gpio_set_pull_mode(
                        Pin,
                        esp_idf_sys::gpio_pull_mode_t_GPIO_PULLUP_PULLDOWN as u32,
                    );
                },
            }
        }

        Ok(size_of::<Pin_data_type>())
    }

    fn get_size(&self) -> file_system::Result<usize> {
        Ok(size_of::<Pin_data_type>())
    }

    fn Set_position(&self, _: &file_system::Position_type) -> file_system::Result<usize> {
        Ok(0)
    }

    fn Flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
