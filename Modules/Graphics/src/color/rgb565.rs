use core::fmt::Debug;

use super::{Color_ARGB8888_type, Color_RGBA8888_type};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color_RGB565_type(u16);

impl Debug for Color_RGB565_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Color_RGB565_type")
            .field("Red", &self.get_red())
            .field("Green", &self.get_green())
            .field("Blue", &self.get_blue())
            .finish()
    }
}

impl Color_RGB565_type {
    pub const fn new(red: u8, Green: u8, Blue: u8) -> Self {
        Self(0).set_red(red).set_green(Green).set_blue(Blue)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }

    pub const fn get_red(&self) -> u8 {
        ((self.0 >> 11) & 0b11111) as u8
    }

    pub const fn get_green(&self) -> u8 {
        ((self.0 >> 5) & 0b111111) as u8
    }

    pub const fn get_blue(&self) -> u8 {
        (self.0 & 0b11111) as u8
    }

    pub const fn set_red(mut self, Value: u8) -> Self {
        let value = Value & 0b11111; // 5 bits
        self.0 = (self.0 & !(0b11111 << 11)) | ((value as u16) << 11);
        self
    }

    pub const fn set_green(mut self, Value: u8) -> Self {
        let value = Value & 0b111111; // 6 bits
        self.0 = (self.0 & !(0b111111 << 5)) | ((value as u16) << 5);
        self
    }

    pub const fn set_blue(mut self, Value: u8) -> Self {
        let value = Value & 0b11111; // 5 bits
        self.0 = (self.0 & !0b11111) | (value as u16);
        self
    }
}

impl From<Color_RGB565_type> for u16 {
    fn from(value: Color_RGB565_type) -> u16 {
        value.0
    }
}

impl From<u16> for Color_RGB565_type {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Color_ARGB8888_type> for Color_RGB565_type {
    fn from(value: Color_ARGB8888_type) -> Self {
        Self::new(
            value.get_red() >> 3,
            value.get_green() >> 2,
            value.get_blue() >> 3,
        )
    }
}

impl From<Color_RGBA8888_type> for Color_RGB565_type {
    fn from(value: Color_RGBA8888_type) -> Self {
        Self::new(
            value.get_red() >> 3,
            value.get_green() >> 2,
            value.get_blue() >> 3,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb565() {
        let Color = Color_RGB565_type::new(255, 255, 255);
        assert_eq!(Color.get_red(), 0b11111);
        assert_eq!(Color.get_green(), 0b111111);
        assert_eq!(Color.get_blue(), 0b11111);
        assert_eq!(Color.0, 0xFFFF);

        let Color = Color_RGB565_type::new(255, 0, 0);
        assert_eq!(Color.get_red(), 0b11111);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0xF800);

        let Color = Color_RGB565_type::new(0, 255, 0);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0b111111);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0x07E0);

        let Color = Color_RGB565_type::new(0, 0, 255);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0b11111);
        assert_eq!(Color.0, 0x00_1F);

        let Color = Color_RGB565_type::new(0, 0, 0);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0);
    }
}
