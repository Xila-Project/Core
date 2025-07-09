use crate::lvgl;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color_RGB888_type {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color_RGB888_type {
    pub const WHITE: Color_RGB888_type = Color_RGB888_type::New(0xFF, 0xFF, 0xFF);
    pub const BLACK: Color_RGB888_type = Color_RGB888_type::New(0x00, 0x00, 0x00);

    pub const fn New(Red: u8, Green: u8, Blue: u8) -> Self {
        Self {
            red: Red,
            green: Green,
            blue: Blue,
        }
    }

    pub const fn get_red(&self) -> u8 {
        self.red
    }

    pub const fn get_green(&self) -> u8 {
        self.green
    }

    pub const fn get_blue(&self) -> u8 {
        self.blue
    }

    pub const fn Set_red(mut self, Value: u8) -> Self {
        self.red = Value;
        self
    }

    pub const fn Set_green(mut self, Value: u8) -> Self {
        self.green = Value;
        self
    }

    pub const fn Set_blue(mut self, Value: u8) -> Self {
        self.blue = Value;
        self
    }

    pub const fn From_LVGL_color(Value: lvgl::lv_color_t) -> Self {
        Self::New(Value.red, Value.green, Value.blue)
    }

    pub const fn into_lvgl_color(self) -> lvgl::lv_color_t {
        lvgl::lv_color_t {
            red: self.get_red(),
            green: self.get_green(),
            blue: self.get_blue(),
        }
    }
}

impl From<Color_RGB888_type> for lvgl::lv_color_t {
    fn from(value: Color_RGB888_type) -> Self {
        value.into_lvgl_color()
    }
}

impl From<lvgl::lv_color_t> for Color_RGB888_type {
    fn from(value: lvgl::lv_color_t) -> Self {
        Color_RGB888_type::From_LVGL_color(value)
    }
}
