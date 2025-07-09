use crate::Color_type;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Hue_type {
    Red = 0,
    Pink = 1,
    Purple = 2,
    Deep_purple = 3,
    Indigo = 4,
    Blue = 5,
    Light_blue = 6,
    Cyan = 7,
    Teal = 8,
    Green = 9,
    Light_green = 10,
    Lime = 11,
    Yellow = 12,
    Amber = 13,
    Orange = 14,
    Deep_orange = 15,
    Brown = 16,
    Blue_grey = 17,
    Grey = 18,
}

// Material Design Color Palette Constants
// Each color has 10 tones: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900
// Tone 500 is the main color
const MATERIAL_COLORS: [[Color_type; 10]; 19] = [
    // Red
    [
        Color_type::new(0xFF, 0xEB, 0xEE), // 50
        Color_type::new(0xFF, 0xCD, 0xD2), // 100
        Color_type::new(0xEF, 0x9A, 0x9A), // 200
        Color_type::new(0xE5, 0x73, 0x73), // 300
        Color_type::new(0xEF, 0x53, 0x50), // 400
        Color_type::new(0xF4, 0x43, 0x36), // 500 (main)
        Color_type::new(0xE5, 0x39, 0x35), // 600
        Color_type::new(0xD3, 0x2F, 0x2F), // 700
        Color_type::new(0xC6, 0x28, 0x28), // 800
        Color_type::new(0xB7, 0x1C, 0x1C), // 900
    ],
    // Pink
    [
        Color_type::new(0xFC, 0xE4, 0xEC), // 50
        Color_type::new(0xF8, 0xBB, 0xD9), // 100
        Color_type::new(0xF4, 0x8F, 0xB1), // 200
        Color_type::new(0xF0, 0x62, 0x92), // 300
        Color_type::new(0xEC, 0x40, 0x7A), // 400
        Color_type::new(0xE9, 0x1E, 0x63), // 500 (main)
        Color_type::new(0xD8, 0x1B, 0x60), // 600
        Color_type::new(0xC2, 0x18, 0x5B), // 700
        Color_type::new(0xAD, 0x14, 0x57), // 800
        Color_type::new(0x88, 0x0E, 0x4F), // 900
    ],
    // Purple
    [
        Color_type::new(0xF3, 0xE5, 0xF5), // 50
        Color_type::new(0xE1, 0xBE, 0xE7), // 100
        Color_type::new(0xCE, 0x93, 0xD8), // 200
        Color_type::new(0xBA, 0x68, 0xC8), // 300
        Color_type::new(0xAB, 0x47, 0xBC), // 400
        Color_type::new(0x9C, 0x27, 0xB0), // 500 (main)
        Color_type::new(0x8E, 0x24, 0xAA), // 600
        Color_type::new(0x7B, 0x1F, 0xA2), // 700
        Color_type::new(0x6A, 0x1B, 0x9A), // 800
        Color_type::new(0x4A, 0x14, 0x8C), // 900
    ],
    // Deep Purple
    [
        Color_type::new(0xED, 0xE7, 0xF6), // 50
        Color_type::new(0xD1, 0xC4, 0xE9), // 100
        Color_type::new(0xB3, 0x9D, 0xDB), // 200
        Color_type::new(0x95, 0x75, 0xCD), // 300
        Color_type::new(0x7E, 0x57, 0xC2), // 400
        Color_type::new(0x67, 0x3A, 0xB7), // 500 (main)
        Color_type::new(0x5E, 0x35, 0xB1), // 600
        Color_type::new(0x51, 0x2D, 0xA8), // 700
        Color_type::new(0x45, 0x27, 0xA0), // 800
        Color_type::new(0x31, 0x1B, 0x92), // 900
    ],
    // Indigo
    [
        Color_type::new(0xE8, 0xEA, 0xF6), // 50
        Color_type::new(0xC5, 0xCA, 0xE9), // 100
        Color_type::new(0x9F, 0xA8, 0xDA), // 200
        Color_type::new(0x79, 0x86, 0xCB), // 300
        Color_type::new(0x5C, 0x6B, 0xC0), // 400
        Color_type::new(0x3F, 0x51, 0xB5), // 500 (main)
        Color_type::new(0x39, 0x49, 0xAB), // 600
        Color_type::new(0x30, 0x3F, 0x9F), // 700
        Color_type::new(0x28, 0x35, 0x93), // 800
        Color_type::new(0x1A, 0x23, 0x7E), // 900
    ],
    // Blue
    [
        Color_type::new(0xE3, 0xF2, 0xFD), // 50
        Color_type::new(0xBB, 0xDE, 0xFB), // 100
        Color_type::new(0x90, 0xCA, 0xF9), // 200
        Color_type::new(0x64, 0xB5, 0xF6), // 300
        Color_type::new(0x42, 0xA5, 0xF5), // 400
        Color_type::new(0x21, 0x96, 0xF3), // 500 (main)
        Color_type::new(0x1E, 0x88, 0xE5), // 600
        Color_type::new(0x19, 0x76, 0xD2), // 700
        Color_type::new(0x15, 0x65, 0xC0), // 800
        Color_type::new(0x0D, 0x47, 0xA1), // 900
    ],
    // Light Blue
    [
        Color_type::new(0xE1, 0xF5, 0xFE), // 50
        Color_type::new(0xB3, 0xE5, 0xFC), // 100
        Color_type::new(0x81, 0xD4, 0xFA), // 200
        Color_type::new(0x4F, 0xC3, 0xF7), // 300
        Color_type::new(0x29, 0xB6, 0xF6), // 400
        Color_type::new(0x03, 0xA9, 0xF4), // 500 (main)
        Color_type::new(0x03, 0x9B, 0xE5), // 600
        Color_type::new(0x02, 0x88, 0xD1), // 700
        Color_type::new(0x02, 0x77, 0xBD), // 800
        Color_type::new(0x01, 0x57, 0x9B), // 900
    ],
    // Cyan
    [
        Color_type::new(0xE0, 0xF7, 0xFA), // 50
        Color_type::new(0xB2, 0xEB, 0xF2), // 100
        Color_type::new(0x80, 0xDE, 0xEA), // 200
        Color_type::new(0x4D, 0xD0, 0xE1), // 300
        Color_type::new(0x26, 0xC6, 0xDA), // 400
        Color_type::new(0x00, 0xBC, 0xD4), // 500 (main)
        Color_type::new(0x00, 0xAC, 0xC1), // 600
        Color_type::new(0x00, 0x97, 0xA7), // 700
        Color_type::new(0x00, 0x83, 0x8F), // 800
        Color_type::new(0x00, 0x60, 0x64), // 900
    ],
    // Teal
    [
        Color_type::new(0xE0, 0xF2, 0xF1), // 50
        Color_type::new(0xB2, 0xDF, 0xDB), // 100
        Color_type::new(0x80, 0xCB, 0xC4), // 200
        Color_type::new(0x4D, 0xB6, 0xAC), // 300
        Color_type::new(0x26, 0xA6, 0x9A), // 400
        Color_type::new(0x00, 0x96, 0x88), // 500 (main)
        Color_type::new(0x00, 0x89, 0x7B), // 600
        Color_type::new(0x00, 0x79, 0x6B), // 700
        Color_type::new(0x00, 0x69, 0x5C), // 800
        Color_type::new(0x00, 0x4D, 0x40), // 900
    ],
    // Green
    [
        Color_type::new(0xE8, 0xF5, 0xE9), // 50
        Color_type::new(0xC8, 0xE6, 0xC9), // 100
        Color_type::new(0xA5, 0xD6, 0xA7), // 200
        Color_type::new(0x81, 0xC7, 0x84), // 300
        Color_type::new(0x66, 0xBB, 0x6A), // 400
        Color_type::new(0x4C, 0xAF, 0x50), // 500 (main)
        Color_type::new(0x43, 0xA0, 0x47), // 600
        Color_type::new(0x38, 0x8E, 0x3C), // 700
        Color_type::new(0x2E, 0x7D, 0x32), // 800
        Color_type::new(0x1B, 0x5E, 0x20), // 900
    ],
    // Light Green
    [
        Color_type::new(0xF1, 0xF8, 0xE9), // 50
        Color_type::new(0xDC, 0xED, 0xC8), // 100
        Color_type::new(0xC5, 0xE1, 0xA5), // 200
        Color_type::new(0xAE, 0xD5, 0x81), // 300
        Color_type::new(0x9C, 0xCC, 0x65), // 400
        Color_type::new(0x8B, 0xC3, 0x4A), // 500 (main)
        Color_type::new(0x7C, 0xB3, 0x42), // 600
        Color_type::new(0x68, 0x9F, 0x38), // 700
        Color_type::new(0x55, 0x8B, 0x2F), // 800
        Color_type::new(0x33, 0x69, 0x1E), // 900
    ],
    // Lime
    [
        Color_type::new(0xF9, 0xFB, 0xE7), // 50
        Color_type::new(0xF0, 0xF4, 0xC3), // 100
        Color_type::new(0xE6, 0xEE, 0x9C), // 200
        Color_type::new(0xDC, 0xE7, 0x75), // 300
        Color_type::new(0xD4, 0xE1, 0x57), // 400
        Color_type::new(0xCD, 0xDC, 0x39), // 500 (main)
        Color_type::new(0xC0, 0xCA, 0x33), // 600
        Color_type::new(0xAF, 0xB4, 0x2B), // 700
        Color_type::new(0x9E, 0x9D, 0x24), // 800
        Color_type::new(0x82, 0x77, 0x17), // 900
    ],
    // Yellow
    [
        Color_type::new(0xFF, 0xFD, 0xE7), // 50
        Color_type::new(0xFF, 0xF9, 0xC4), // 100
        Color_type::new(0xFF, 0xF5, 0x9D), // 200
        Color_type::new(0xFF, 0xF1, 0x76), // 300
        Color_type::new(0xFF, 0xEE, 0x58), // 400
        Color_type::new(0xFF, 0xEB, 0x3B), // 500 (main)
        Color_type::new(0xFD, 0xD8, 0x35), // 600
        Color_type::new(0xFB, 0xC0, 0x2D), // 700
        Color_type::new(0xF9, 0xA8, 0x25), // 800
        Color_type::new(0xF5, 0x7F, 0x17), // 900
    ],
    // Amber
    [
        Color_type::new(0xFF, 0xF8, 0xE1), // 50
        Color_type::new(0xFF, 0xEC, 0xB3), // 100
        Color_type::new(0xFF, 0xE0, 0x82), // 200
        Color_type::new(0xFF, 0xD5, 0x4F), // 300
        Color_type::new(0xFF, 0xCA, 0x28), // 400
        Color_type::new(0xFF, 0xC1, 0x07), // 500 (main)
        Color_type::new(0xFF, 0xB3, 0x00), // 600
        Color_type::new(0xFF, 0xA0, 0x00), // 700
        Color_type::new(0xFF, 0x8F, 0x00), // 800
        Color_type::new(0xFF, 0x6F, 0x00), // 900
    ],
    // Orange
    [
        Color_type::new(0xFF, 0xF3, 0xE0), // 50
        Color_type::new(0xFF, 0xE0, 0xB2), // 100
        Color_type::new(0xFF, 0xCC, 0x80), // 200
        Color_type::new(0xFF, 0xB7, 0x4D), // 300
        Color_type::new(0xFF, 0xA7, 0x26), // 400
        Color_type::new(0xFF, 0x98, 0x00), // 500 (main)
        Color_type::new(0xFB, 0x8C, 0x00), // 600
        Color_type::new(0xF5, 0x7C, 0x00), // 700
        Color_type::new(0xEF, 0x6C, 0x00), // 800
        Color_type::new(0xE6, 0x51, 0x00), // 900
    ],
    // Deep Orange
    [
        Color_type::new(0xFB, 0xE9, 0xE7), // 50
        Color_type::new(0xFF, 0xCC, 0xBC), // 100
        Color_type::new(0xFF, 0xAB, 0x91), // 200
        Color_type::new(0xFF, 0x8A, 0x65), // 300
        Color_type::new(0xFF, 0x70, 0x43), // 400
        Color_type::new(0xFF, 0x57, 0x22), // 500 (main)
        Color_type::new(0xF4, 0x51, 0x1E), // 600
        Color_type::new(0xE6, 0x4A, 0x19), // 700
        Color_type::new(0xD8, 0x43, 0x15), // 800
        Color_type::new(0xBF, 0x36, 0x0C), // 900
    ],
    // Brown
    [
        Color_type::new(0xEF, 0xEB, 0xE9), // 50
        Color_type::new(0xD7, 0xCC, 0xC8), // 100
        Color_type::new(0xBC, 0xAA, 0xA4), // 200
        Color_type::new(0xA1, 0x88, 0x7F), // 300
        Color_type::new(0x8D, 0x6E, 0x63), // 400
        Color_type::new(0x79, 0x55, 0x48), // 500 (main)
        Color_type::new(0x6D, 0x4C, 0x41), // 600
        Color_type::new(0x5D, 0x40, 0x37), // 700
        Color_type::new(0x4E, 0x34, 0x2E), // 800
        Color_type::new(0x3E, 0x27, 0x23), // 900
    ],
    // Blue Grey
    [
        Color_type::new(0xEC, 0xEF, 0xF1), // 50
        Color_type::new(0xCF, 0xD8, 0xDC), // 100
        Color_type::new(0xB0, 0xBE, 0xC5), // 200
        Color_type::new(0x90, 0xA4, 0xAE), // 300
        Color_type::new(0x78, 0x90, 0x9C), // 400
        Color_type::new(0x60, 0x7D, 0x8B), // 500 (main)
        Color_type::new(0x54, 0x6E, 0x7A), // 600
        Color_type::new(0x45, 0x5A, 0x64), // 700
        Color_type::new(0x37, 0x47, 0x4F), // 800
        Color_type::new(0x26, 0x32, 0x38), // 900
    ],
    // Grey
    [
        Color_type::new(0xFA, 0xFA, 0xFA), // 50
        Color_type::new(0xF5, 0xF5, 0xF5), // 100
        Color_type::new(0xEE, 0xEE, 0xEE), // 200
        Color_type::new(0xE0, 0xE0, 0xE0), // 300
        Color_type::new(0xBD, 0xBD, 0xBD), // 400
        Color_type::new(0x9E, 0x9E, 0x9E), // 500 (main)
        Color_type::new(0x75, 0x75, 0x75), // 600
        Color_type::new(0x61, 0x61, 0x61), // 700
        Color_type::new(0x42, 0x42, 0x42), // 800
        Color_type::new(0x21, 0x21, 0x21), // 900
    ],
];

impl Hue_type {
    /// Get the index of this hue in the material colors array
    const fn get_color_index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tone_type {
    Tone_50 = 0,
    Tone_100 = 1,
    Tone_200 = 2,
    Tone_300 = 3,
    Tone_400 = 4,
    Tone_500 = 5,
    Tone_600 = 6,
    Tone_700 = 7,
    Tone_800 = 8,
    Tone_900 = 9,
}

impl Tone_type {
    pub const MAIN: Tone_type = Tone_type::Tone_500;

    /// Get the index of this tone in the material colors array
    const fn get_tone_index(self) -> usize {
        self as usize
    }
}

/// Get a Material Design color from the palette
/// This function is const-compliant and doesn't rely on LVGL
pub const fn get(hue: Hue_type, tone: Tone_type) -> Color_type {
    let color_index = hue.get_color_index();
    let tone_index = tone.get_tone_index();
    MATERIAL_COLORS[color_index][tone_index]
}
