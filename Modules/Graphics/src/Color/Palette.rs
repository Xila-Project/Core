use crate::LVGL;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color_type {
    Red,
    Pink,
    Purple,
    Deep_purple,
    Indigo,
    Blue,
    Light_blue,
    Cyan,
    Teal,
    Green,
    Light_green,
    Lime,
    Yellow,
    Amber,
    Orange,
    Deep_orange,
    Brown,
    Blue_grey,
    Grey,
}

impl Color_type {
    fn Into_LVGL_palette_color(self) -> u32 {
        match self {
            Color_type::Red => LVGL::lv_palette_t_LV_PALETTE_RED,
            Color_type::Pink => LVGL::lv_palette_t_LV_PALETTE_PINK,
            Color_type::Purple => LVGL::lv_palette_t_LV_PALETTE_PURPLE,
            Color_type::Deep_purple => LVGL::lv_palette_t_LV_PALETTE_DEEP_PURPLE,
            Color_type::Indigo => LVGL::lv_palette_t_LV_PALETTE_INDIGO,
            Color_type::Blue => LVGL::lv_palette_t_LV_PALETTE_BLUE,
            Color_type::Light_blue => LVGL::lv_palette_t_LV_PALETTE_LIGHT_BLUE,
            Color_type::Cyan => LVGL::lv_palette_t_LV_PALETTE_CYAN,
            Color_type::Teal => LVGL::lv_palette_t_LV_PALETTE_TEAL,
            Color_type::Green => LVGL::lv_palette_t_LV_PALETTE_GREEN,
            Color_type::Light_green => LVGL::lv_palette_t_LV_PALETTE_LIGHT_GREEN,
            Color_type::Lime => LVGL::lv_palette_t_LV_PALETTE_LIME,
            Color_type::Yellow => LVGL::lv_palette_t_LV_PALETTE_YELLOW,
            Color_type::Amber => LVGL::lv_palette_t_LV_PALETTE_AMBER,
            Color_type::Orange => LVGL::lv_palette_t_LV_PALETTE_ORANGE,
            Color_type::Deep_orange => LVGL::lv_palette_t_LV_PALETTE_DEEP_ORANGE,
            Color_type::Brown => LVGL::lv_palette_t_LV_PALETTE_BROWN,
            Color_type::Blue_grey => LVGL::lv_palette_t_LV_PALETTE_BLUE_GREY,
            Color_type::Grey => LVGL::lv_palette_t_LV_PALETTE_GREY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level_type {
    Tone_50 = -5,
    Tone_100 = -4,
    Tone_200 = -3,
    Tone_300 = -2,
    Tone_400 = -1,
    Tone_500 = 0,
    Tone_600 = 1,
    Tone_700 = 2,
    Tone_800 = 3,
    Tone_900 = 4,
}

impl Level_type {
    pub const Main: Level_type = Level_type::Tone_500;
}

pub fn Get(Color: Color_type, Level: Level_type) -> super::Color_type {
    let Color = Color.Into_LVGL_palette_color();

    let Level = Level as i8;

    let Color = unsafe {
        match Level {
            -5..=-1 => LVGL::lv_palette_lighten(Color, -Level as u8),
            0 => LVGL::lv_palette_main(Color),
            1..=4 => LVGL::lv_palette_darken(Color, Level as u8),
            _ => unreachable!(),
        }
    };

    super::Color_type::From_LVGL_color(Color)
}
