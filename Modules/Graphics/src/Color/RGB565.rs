use core::fmt::Debug;

use super::{Color_ARGB8888_type, Color_RGBA8888_type};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color_RGB565_type(u16);

impl Debug for Color_RGB565_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Formatter
            .debug_struct("Color_RGB565_type")
            .field("Red", &self.Get_red())
            .field("Green", &self.Get_green())
            .field("Blue", &self.Get_blue())
            .finish()
    }
}

impl Color_RGB565_type {
    pub const fn New(Red: u8, Green: u8, Blue: u8) -> Self {
        Self(0).Set_red(Red).Set_green(Green).Set_blue(Blue)
    }

    pub const fn As_u16(self) -> u16 {
        self.0
    }

    pub const fn Get_red(&self) -> u8 {
        ((self.0 >> 11) & 0b11111) as u8
    }

    pub const fn Get_green(&self) -> u8 {
        ((self.0 >> 5) & 0b111111) as u8
    }

    pub const fn Get_blue(&self) -> u8 {
        (self.0 & 0b11111) as u8
    }

    pub const fn Set_red(mut self, Value: u8) -> Self {
        let Value = Value & 0b11111; // 5 bits
        self.0 = (self.0 & !(0b11111 << 11)) | ((Value as u16) << 11);
        self
    }

    pub const fn Set_green(mut self, Value: u8) -> Self {
        let Value = Value & 0b111111; // 6 bits
        self.0 = (self.0 & !(0b111111 << 5)) | ((Value as u16) << 5);
        self
    }

    pub const fn Set_blue(mut self, Value: u8) -> Self {
        let Value = Value & 0b11111; // 5 bits
        self.0 = (self.0 & !0b11111) | (Value as u16);
        self
    }
}

impl From<Color_RGB565_type> for u16 {
    fn from(Value: Color_RGB565_type) -> u16 {
        Value.0
    }
}

impl From<u16> for Color_RGB565_type {
    fn from(Value: u16) -> Self {
        Self(Value)
    }
}

impl From<Color_ARGB8888_type> for Color_RGB565_type {
    fn from(Value: Color_ARGB8888_type) -> Self {
        Self::New(
            Value.Get_red() >> 3,
            Value.Get_green() >> 2,
            Value.Get_blue() >> 3,
        )
    }
}

impl From<Color_RGBA8888_type> for Color_RGB565_type {
    fn from(Value: Color_RGBA8888_type) -> Self {
        Self::New(
            Value.Get_red() >> 3,
            Value.Get_green() >> 2,
            Value.Get_blue() >> 3,
        )
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_RGB565() {
        let Color = Color_RGB565_type::New(255, 255, 255);
        assert_eq!(Color.Get_red(), 0b11111);
        assert_eq!(Color.Get_green(), 0b111111);
        assert_eq!(Color.Get_blue(), 0b11111);
        assert_eq!(Color.0, 0xFFFF);

        let Color = Color_RGB565_type::New(255, 0, 0);
        assert_eq!(Color.Get_red(), 0b11111);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0xF800);

        let Color = Color_RGB565_type::New(0, 255, 0);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0b111111);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0x07E0);

        let Color = Color_RGB565_type::New(0, 0, 255);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0b11111);
        assert_eq!(Color.0, 0x00_1F);

        let Color = Color_RGB565_type::New(0, 0, 0);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0);
    }
}
