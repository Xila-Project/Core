#[cfg(feature = "ARGB8888")]
pub type Color_type = Color_ARGB8888_type;

#[cfg(feature = "RGB565")]
pub type Color_type = Color_RGB565_type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color_RGB565_type(u16);

impl Color_RGB565_type {
    pub fn New(Red: u8, Green: u8, Blue: u8) -> Self {
        Self(0).Set_red(Red).Set_green(Green).Set_blue(Blue)
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

    pub fn Set_red(mut self, Value: u8) -> Self {
        let Value = Value & 0b11111; // 5 bits
        self.0 = (self.0 & !(0b11111 << 11)) | ((Value as u16) << 11);
        self
    }

    pub fn Set_green(mut self, Value: u8) -> Self {
        let Value = Value & 0b111111; // 6 bits
        self.0 = (self.0 & !(0b111111 << 5)) | ((Value as u16) << 5);
        self
    }

    pub fn Set_blue(mut self, Value: u8) -> Self {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color_ARGB8888_type(u32);

impl Color_ARGB8888_type {
    pub fn New(Alpha: u8, Red: u8, Green: u8, Blue: u8) -> Self {
        Self(0)
            .Set_alpha(Alpha)
            .Set_red(Red)
            .Set_green(Green)
            .Set_blue(Blue)
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

    pub fn Set_alpha(mut self, Value: u8) -> Self {
        self.0 = (self.0 & !(0b1111_1111 << 24)) | ((Value as u32) << 24);
        self
    }

    pub fn Set_red(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFF00_FFFF) | ((Value as u32) << 16);
        self
    }

    pub fn Set_green(mut self, Value: u8) -> Self {
        self.0 = (self.0 & 0xFFFF_00FF) | ((Value as u32) << 8);
        self
    }

    pub fn Set_blue(mut self, Value: u8) -> Self {
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
        Self::New(
            0xFF,
            Value.Get_red() << 3,
            Value.Get_green() << 2,
            Value.Get_blue() << 3,
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

    #[test]
    fn Test_Conversion() {
        let Color = Color_RGB565_type::New(255, 255, 255);
        let Color = Color_ARGB8888_type::from(Color);
        assert_eq!(Color.Get_alpha(), 0xFF);
        assert_eq!(Color.Get_red(), 248);
        assert_eq!(Color.Get_green(), 252);
        assert_eq!(Color.Get_blue(), 248);

        let Color = Color_ARGB8888_type::New(255, 255, 255, 255);
        let Color = Color_RGB565_type::from(Color);
        assert_eq!(Color.Get_red(), 0b11111);
        assert_eq!(Color.Get_green(), 0b111111);
        assert_eq!(Color.Get_blue(), 0b11111);
    }
}
