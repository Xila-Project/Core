use crate::lvgl;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color_RGB888_type {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color_RGB888_type {
    pub const WHITE: Color_RGB888_type = Color_RGB888_type::new(0xFF, 0xFF, 0xFF);
    pub const BLACK: Color_RGB888_type = Color_RGB888_type::new(0x00, 0x00, 0x00);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
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

    pub const fn set_red(mut self, value: u8) -> Self {
        self.red = value;
        self
    }

    pub const fn set_green(mut self, value: u8) -> Self {
        self.green = value;
        self
    }

    pub const fn set_blue(mut self, value: u8) -> Self {
        self.blue = value;
        self
    }

    pub const fn from_lvgl_color(value: lvgl::lv_color_t) -> Self {
        Self::new(value.red, value.green, value.blue)
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
        Color_RGB888_type::from_lvgl_color(value)
    }
}
