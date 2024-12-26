use std::fmt::Debug;

use super::{Color_RGB565_type, Color_RGBA8888_type};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color_ARGB8888_type(u32);

impl Debug for Color_ARGB8888_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Color_ARGB8888_type")
            .field("Alpha", &self.Get_alpha())
            .field("Red", &self.Get_red())
            .field("Green", &self.Get_green())
            .field("Blue", &self.Get_blue())
            .finish()
    }
}

impl Color_ARGB8888_type {
    pub const fn New(Alpha: u8, Red: u8, Green: u8, Blue: u8) -> Self {
        Self(0)
            .Set_alpha(Alpha)
            .Set_red(Red)
            .Set_green(Green)
            .Set_blue(Blue)
    }

    pub const fn From_RGB565(Value: Color_RGB565_type) -> Self {
        Self::New(
            0xFF,
            Value.Get_red() << 3 | Value.Get_red() >> 2,
            Value.Get_green() << 2 | Value.Get_green() >> 4,
            Value.Get_blue() << 3 | Value.Get_blue() >> 2,
        )
    }

    pub const fn As_u32(self) -> u32 {
        self.0
    }

    pub const fn Get_alpha(&self) -> u8 {
        ((self.0 >> 24) & 0b1111_1111) as u8
    }

    pub const fn Get_red(&self) -> u8 {
        ((self.0 >> 16) & 0b1111_1111) as u8
    }

    pub const fn Get_green(&self) -> u8 {
        ((self.0 >> 8) & 0b1111_1111) as u8
    }

    pub const fn Get_blue(&self) -> u8 {
        ((self.0) & 0b1111_1111) as u8
    }

    pub const fn Set_alpha(mut self, Value: u8) -> Self {
        self.0 = (self.0 & !(0b1111_1111 << 24)) | ((Value as u32) << 24);
        self
    }

    pub const fn Set_red(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFF00_FFFF) | ((Value as u32) << 16);
        self
    }

    pub const fn Set_green(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFFFF_00FF) | ((Value as u32) << 8);
        self
    }

    pub const fn Set_blue(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFFFF_FF00) | (Value as u32);
        self
    }
}

impl From<Color_ARGB8888_type> for u32 {
    fn from(Value: Color_ARGB8888_type) -> u32 {
        Value.0
    }
}

impl From<u32> for Color_ARGB8888_type {
    fn from(Value: u32) -> Self {
        Self(Value)
    }
}

impl From<Color_RGB565_type> for Color_ARGB8888_type {
    fn from(Value: Color_RGB565_type) -> Self {
        Self::From_RGB565(Value)
    }
}

impl From<Color_RGBA8888_type> for Color_ARGB8888_type {
    fn from(Value: Color_RGBA8888_type) -> Self {
        Self::New(
            Value.Get_alpha(),
            Value.Get_red(),
            Value.Get_green(),
            Value.Get_blue(),
        )
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_ARGB8888() {
        let Color = Color_ARGB8888_type::New(255, 255, 255, 255);
        assert_eq!(Color.Get_alpha(), 0xFF);
        assert_eq!(Color.Get_red(), 0xFF);
        assert_eq!(Color.Get_green(), 0xFF);
        assert_eq!(Color.Get_blue(), 0xFF);
        assert_eq!(Color.0, 0xFFFF_FFFF);

        let Color = Color_ARGB8888_type::New(255, 0, 0, 0);
        assert_eq!(Color.Get_alpha(), 0xFF);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0xFF00_0000);

        let Color = Color_ARGB8888_type::New(0, 255, 0, 0);
        assert_eq!(Color.Get_alpha(), 0);
        assert_eq!(Color.Get_red(), 0xFF);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0x00FF_0000);

        let Color = Color_ARGB8888_type::New(0, 0, 255, 0);
        assert_eq!(Color.Get_alpha(), 0);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0xFF);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0x0000_FF00);

        let Color = Color_ARGB8888_type::New(0, 0, 0, 255);
        assert_eq!(Color.Get_alpha(), 0);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0xFF);
        assert_eq!(Color.0, 0x0000_00FF);

        let Color = Color_ARGB8888_type::New(0, 0, 0, 0);
        assert_eq!(Color.Get_alpha(), 0);
        assert_eq!(Color.Get_red(), 0);
        assert_eq!(Color.Get_green(), 0);
        assert_eq!(Color.Get_blue(), 0);
        assert_eq!(Color.0, 0);
    }
}
