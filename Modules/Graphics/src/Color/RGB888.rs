use crate::LVGL;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color_RGB888_type {
    Red: u8,
    Green: u8,
    Blue: u8,
}

impl Color_RGB888_type {
    pub const White: Color_RGB888_type = Color_RGB888_type::New(0xFF, 0xFF, 0xFF);
    pub const Black: Color_RGB888_type = Color_RGB888_type::New(0x00, 0x00, 0x00);

    pub const fn New(Red: u8, Green: u8, Blue: u8) -> Self {
        Self { Red, Green, Blue }
    }

    pub const fn Get_red(&self) -> u8 {
        self.Red
    }

    pub const fn Get_green(&self) -> u8 {
        self.Green
    }

    pub const fn Get_blue(&self) -> u8 {
        self.Blue
    }

    pub const fn Set_red(mut self, Value: u8) -> Self {
        self.Red = Value;
        self
    }

    pub const fn Set_green(mut self, Value: u8) -> Self {
        self.Green = Value;
        self
    }

    pub const fn Set_blue(mut self, Value: u8) -> Self {
        self.Blue = Value;
        self
    }

    pub const fn From_LVGL_color(Value: LVGL::lv_color_t) -> Self {
        Self::New(Value.red, Value.green, Value.blue)
    }

    pub const fn Into_LVGL_color(self) -> LVGL::lv_color_t {
        LVGL::lv_color_t {
            red: self.Get_red(),
            green: self.Get_green(),
            blue: self.Get_blue(),
        }
    }
}

impl From<Color_RGB888_type> for LVGL::lv_color_t {
    fn from(Value: Color_RGB888_type) -> Self {
        Value.Into_LVGL_color()
    }
}

impl From<LVGL::lv_color_t> for Color_RGB888_type {
    fn from(Value: LVGL::lv_color_t) -> Self {
        Color_RGB888_type::From_LVGL_color(Value)
    }
}
