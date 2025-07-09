use core::fmt::Debug;

use super::{Color_RGB565_type, Color_RGBA8888_type};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color_ARGB8888_type(u32);

impl Debug for Color_ARGB8888_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Color_ARGB8888_type")
            .field("Alpha", &self.get_alpha())
            .field("Red", &self.get_red())
            .field("Green", &self.get_green())
            .field("Blue", &self.get_blue())
            .finish()
    }
}

impl Color_ARGB8888_type {
    pub const fn new(alpha: u8, Red: u8, Green: u8, Blue: u8) -> Self {
        Self(0)
            .set_alpha(alpha)
            .set_red(Red)
            .set_green(Green)
            .set_blue(Blue)
    }

    pub const fn from_rgb565(Value: Color_RGB565_type) -> Self {
        Self::new(
            0xFF,
            Value.get_red() << 3 | Value.get_red() >> 2,
            Value.get_green() << 2 | Value.get_green() >> 4,
            Value.get_blue() << 3 | Value.get_blue() >> 2,
        )
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub const fn get_alpha(&self) -> u8 {
        ((self.0 >> 24) & 0b1111_1111) as u8
    }

    pub const fn get_red(&self) -> u8 {
        ((self.0 >> 16) & 0b1111_1111) as u8
    }

    pub const fn get_green(&self) -> u8 {
        ((self.0 >> 8) & 0b1111_1111) as u8
    }

    pub const fn get_blue(&self) -> u8 {
        ((self.0) & 0b1111_1111) as u8
    }

    pub const fn set_alpha(mut self, Value: u8) -> Self {
        self.0 = (self.0 & !(0b1111_1111 << 24)) | ((Value as u32) << 24);
        self
    }

    pub const fn set_red(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFF00_FFFF) | ((Value as u32) << 16);
        self
    }

    pub const fn set_green(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFFFF_00FF) | ((Value as u32) << 8);
        self
    }

    pub const fn set_blue(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFFFF_FF00) | (Value as u32);
        self
    }
}

impl From<Color_ARGB8888_type> for u32 {
    fn from(value: Color_ARGB8888_type) -> u32 {
        value.0
    }
}

impl From<u32> for Color_ARGB8888_type {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Color_RGB565_type> for Color_ARGB8888_type {
    fn from(value: Color_RGB565_type) -> Self {
        Self::from_rgb565(value)
    }
}

impl From<Color_RGBA8888_type> for Color_ARGB8888_type {
    fn from(value: Color_RGBA8888_type) -> Self {
        Self::new(
            value.get_alpha(),
            value.get_red(),
            value.get_green(),
            value.get_blue(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argb8888() {
        let Color = Color_ARGB8888_type::new(255, 255, 255, 255);
        assert_eq!(Color.get_alpha(), 0xFF);
        assert_eq!(Color.get_red(), 0xFF);
        assert_eq!(Color.get_green(), 0xFF);
        assert_eq!(Color.get_blue(), 0xFF);
        assert_eq!(Color.0, 0xFFFF_FFFF);

        let Color = Color_ARGB8888_type::new(255, 0, 0, 0);
        assert_eq!(Color.get_alpha(), 0xFF);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0xFF00_0000);

        let Color = Color_ARGB8888_type::new(0, 255, 0, 0);
        assert_eq!(Color.get_alpha(), 0);
        assert_eq!(Color.get_red(), 0xFF);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0x00FF_0000);

        let Color = Color_ARGB8888_type::new(0, 0, 255, 0);
        assert_eq!(Color.get_alpha(), 0);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0xFF);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0x0000_FF00);

        let Color = Color_ARGB8888_type::new(0, 0, 0, 255);
        assert_eq!(Color.get_alpha(), 0);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0xFF);
        assert_eq!(Color.0, 0x0000_00FF);

        let Color = Color_ARGB8888_type::new(0, 0, 0, 0);
        assert_eq!(Color.get_alpha(), 0);
        assert_eq!(Color.get_red(), 0);
        assert_eq!(Color.get_green(), 0);
        assert_eq!(Color.get_blue(), 0);
        assert_eq!(Color.0, 0);
    }
}
