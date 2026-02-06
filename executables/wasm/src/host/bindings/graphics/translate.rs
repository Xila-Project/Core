use core::mem::transmute;

use xila::graphics::lvgl::{self, lv_style_value_t};

use crate::host::virtual_machine::{Error, Result, Translator, WasmPointer, WasmUsize};

pub trait TranslateFrom {
    unsafe fn translate_from(wasm_usize: WasmUsize, translator: Translator) -> Result<Self>
    where
        Self: Sized;
}

pub trait TranslateInto: Sized {
    type Output;

    unsafe fn translate_into(self, translator: Translator) -> Result<Self::Output>;
}

macro_rules! implicit_usize_cast {
    ($($t:ty),* $(,)?) => {
        $(
            impl TranslateFrom for $t {
                #[inline]
                unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
                    Ok(wasm_usize as $t)
                }
            }

            impl TranslateInto for $t {
                type Output = Self;
                #[inline]
                unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
                    Ok(self as $t)
                }
            }
        )*
    };
}

macro_rules! implicit_pointer_translation {
    ($($t:ty),* $(,)?) => {
        $(
            impl TranslateFrom for *mut $t {
                #[inline]
                unsafe fn translate_from(wasm_usize: WasmUsize, translator: Translator) -> Result<Self> {
                    let ptr = unsafe { translator.translate_to_host(wasm_usize as WasmPointer, true) };
                    match ptr {
                        Some(p) => Ok(p as *mut $t),
                        None => Err(Error::InvalidPointer),
                    }
                }
            }

            impl TranslateFrom for *const $t {
                #[inline]
                unsafe fn translate_from(wasm_usize: WasmUsize, translator: Translator) -> Result<Self> {
                    let ptr = unsafe { translator.translate_to_host(wasm_usize as WasmPointer, true) };
                    match ptr {
                        Some(p) => Ok(p as *const $t),
                        None => Err(Error::InvalidPointer),
                    }
                }
            }

            impl TranslateInto for *mut $t {
                type Output = WasmUsize;

                #[inline]
                unsafe fn translate_into(self, translator: Translator) -> Result<Self::Output> {
                    let ptr = unsafe { translator.translate_to_guest(self, true) };
                    match ptr {
                        Some(p) => Ok(p as WasmUsize),
                        None => Err(Error::InvalidPointer),
                    }
                }
            }

            impl TranslateInto for *const $t {
                type Output = WasmUsize;

                #[inline]
                unsafe fn translate_into(self, translator: Translator) -> Result<Self::Output> {
                    let ptr = unsafe { translator.translate_to_guest(self as *mut $t, true) };
                    match ptr {
                        Some(p) => Ok(p as WasmUsize),
                        None => Err(Error::InvalidPointer),
                    }
                }
            }
        )*
    };
}

implicit_pointer_translation!(
    lvgl::lv_point_t,
    lvgl::lv_point_precise_t,
    lvgl::lv_style_t,
    lvgl::lv_anim_t,
    lvgl::lv_coord_t,
    lvgl::lv_obj_class_t,
    lvgl::lv_area_t,
    lvgl::lv_style_value_t,
    lvgl::lv_color16_t,
    lvgl::lv_color32_t,
    lvgl::lv_matrix_t,
    lvgl::lv_chart_series_t,
    lvgl::lv_calendar_date_t,
    core::ffi::c_void,
    i8,
    u8,
);

implicit_usize_cast!(u8, u16, u32, usize, i8, i16, i32, isize, f32);

#[cfg(target_pointer_width = "64")]
implicit_usize_cast!(u64, i64, f64);

impl TranslateFrom for bool {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        Ok(wasm_usize != 0)
    }
}

impl TranslateInto for bool {
    type Output = Self;
    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok(self)
    }
}

impl TranslateFrom for () {
    #[inline]
    unsafe fn translate_from(_: WasmUsize, _: Translator) -> Result<Self> {
        Ok(())
    }
}

impl TranslateInto for () {
    type Output = Self;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self> {
        Ok(())
    }
}

impl TranslateFrom for *mut lvgl::lv_obj_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, translator: Translator) -> Result<Self> {
        let ptr = translator.translate_to_host(wasm_usize as WasmPointer, false);
        match ptr {
            Some(p) => Ok(p as *mut lvgl::lv_obj_t),
            None => Err(Error::InvalidPointer),
        }
    }
}

impl TranslateFrom for *const lvgl::lv_obj_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, translator: Translator) -> Result<Self> {
        let ptr = translator.translate_to_host(wasm_usize as WasmPointer, false);
        match ptr {
            Some(p) => Ok(p as *const lvgl::lv_obj_t),
            None => Err(Error::InvalidPointer),
        }
    }
}

impl TranslateInto for *mut lvgl::lv_obj_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, translator: Translator) -> Result<Self::Output> {
        let ptr = translator.translate_to_guest(self, false);
        match ptr {
            Some(p) => Ok(p as WasmUsize),
            None => Err(Error::InvalidPointer),
        }
    }
}

impl TranslateInto for *mut *mut lvgl::lv_obj_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, translator: Translator) -> Result<Self::Output> {
        let ptr = translator.translate_to_guest(self as *mut lvgl::lv_obj_t, false);
        match ptr {
            Some(p) => Ok(p as WasmUsize),
            None => Err(Error::InvalidPointer),
        }
    }
}

// Inlined translations

impl TranslateFrom for lvgl::lv_color_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        Ok(lvgl::lv_color_t {
            blue: wasm_usize as u8,
            green: (wasm_usize >> 8) as u8,
            red: (wasm_usize >> 16) as u8,
        })
    }
}

impl TranslateInto for lvgl::lv_color_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<WasmUsize> {
        Ok((self.red as WasmUsize) << 16
            | (self.green as WasmUsize) << 8
            | (self.blue as WasmUsize))
    }
}

impl TranslateFrom for lvgl::lv_color32_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, translator: Translator) -> Result<Self> {
        Ok(lvgl::lv_color32_t {
            blue: wasm_usize as u8,
            green: (wasm_usize >> 8) as u8,
            red: (wasm_usize >> 16) as u8,
            alpha: (wasm_usize >> 24) as u8,
        })
    }
}

impl TranslateInto for lvgl::lv_color32_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok((self.alpha as WasmUsize) << 24
            | (self.red as WasmUsize) << 16
            | (self.green as WasmUsize) << 8
            | (self.blue as WasmUsize))
    }
}

impl TranslateFrom for lvgl::lv_color_hsv_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        Ok(lvgl::lv_color_hsv_t {
            h: wasm_usize as u16,
            s: (wasm_usize >> 16) as u8,
            v: (wasm_usize >> 24) as u8,
        })
    }
}

impl TranslateInto for lvgl::lv_color_hsv_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok((self.h as WasmUsize) | (self.s as WasmUsize) << 16 | (self.v as WasmUsize) << 24)
    }
}

impl TranslateFrom for lvgl::lv_color16_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        let value = unsafe { transmute::<u16, lvgl::lv_color16_t>(wasm_usize as u16) };
        Ok(value)
    }
}

impl TranslateInto for lvgl::lv_color16_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok(unsafe { transmute::<lvgl::lv_color16_t, u16>(self) as WasmUsize })
    }
}

impl TranslateFrom for lvgl::lv_style_value_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        let value = wasm_usize as *mut lv_style_value_t;
        unsafe { Ok(*value) }
    }
}

const POINT_Y_OFFSET: WasmUsize = size_of::<WasmUsize>() as WasmUsize * 8 / 2;
const POINT_MASK: WasmUsize = (1 << POINT_Y_OFFSET) - 1;

impl TranslateFrom for lvgl::lv_point_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        let x = (wasm_usize & POINT_MASK) as i32;
        let y = (wasm_usize >> POINT_Y_OFFSET) as i32;
        Ok(lvgl::lv_point_t { x, y })
    }
}

impl TranslateInto for lvgl::lv_point_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok((self.y as WasmUsize) << POINT_Y_OFFSET | (self.x as WasmUsize))
    }
}

impl TranslateFrom for lvgl::lv_point_precise_t {
    #[inline]
    unsafe fn translate_from(wasm_usize: WasmUsize, _: Translator) -> Result<Self> {
        let x = (wasm_usize & POINT_MASK) as i32;
        let y = (wasm_usize >> POINT_Y_OFFSET) as i32;
        Ok(lvgl::lv_point_precise_t { x, y })
    }
}

impl TranslateInto for lvgl::lv_point_precise_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok((self.y as WasmUsize) << POINT_Y_OFFSET | (self.x as WasmUsize))
    }
}

impl TranslateInto for lvgl::lv_style_value_t {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, _: Translator) -> Result<Self::Output> {
        Ok(0)
    }
}

impl TranslateInto for *const *const i8 {
    type Output = WasmUsize;

    #[inline]
    unsafe fn translate_into(self, translator: Translator) -> Result<Self::Output> {
        let ptr = translator.translate_to_guest(self as *mut *mut i8, true);
        match ptr {
            Some(p) => Ok(p as WasmUsize),
            None => Err(Error::InvalidPointer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xila::graphics::lvgl;

    fn mock_translator() -> Translator<'static> {
        // Create a mock translator for testing
        // Adjust based on your actual Translator implementation
        unsafe { core::mem::zeroed() }
    }

    #[test]
    fn test_u8_translation() {
        let translator = mock_translator();
        assert_eq!(u8::translate_from(42, translator).unwrap(), 42u8);
        assert_eq!(42u8.translate_into(translator).unwrap(), 42u8);
    }

    #[test]
    fn test_bool_translation() {
        let translator = mock_translator();
        assert_eq!(bool::translate_from(0, translator).unwrap(), false);
        assert_eq!(bool::translate_from(1, translator).unwrap(), true);
        assert_eq!(bool::translate_from(255, translator).unwrap(), true);
        assert_eq!(true.translate_into(translator).unwrap(), true);
        assert_eq!(false.translate_into(translator).unwrap(), false);
    }

    #[test]
    fn test_unit_translation() {
        let translator = mock_translator();
        assert_eq!(<()>::translate_from(0, translator).unwrap(), ());
        assert_eq!(().translate_into(translator).unwrap(), ());
    }

    #[test]
    fn test_lv_color_t_translation() {
        let translator = mock_translator();
        let wasm_value: WasmUsize = 0x00FF8040; // R=255, G=128, B=64
        let color = lvgl::lv_color_t::translate_from(wasm_value, translator).unwrap();
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);

        let result = color.translate_into(translator).unwrap();
        assert_eq!(result, wasm_value);
    }

    #[test]
    fn test_lv_color32_t_translation() {
        let translator = mock_translator();
        let wasm_value: WasmUsize = 0xAAFF8040; // A=170, R=255, G=128, B=64
        let color = lvgl::lv_color32_t::translate_from(wasm_value, translator).unwrap();
        assert_eq!(color.alpha, 170);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);

        let result = color.translate_into(translator).unwrap();
        assert_eq!(result, wasm_value);
    }

    #[test]
    fn test_lv_point_t_translation() {
        let translator = mock_translator();
        let x: i32 = 100;
        let y: i32 = 200;
        let wasm_value: WasmUsize = ((y as WasmUsize) << POINT_Y_OFFSET) | (x as WasmUsize);

        let point = lvgl::lv_point_t::translate_from(wasm_value, translator).unwrap();
        assert_eq!(point.x, x);
        assert_eq!(point.y, y);

        let result = point.translate_into(translator).unwrap();
        assert_eq!(result, wasm_value);
    }

    #[test]
    fn test_lv_point_precise_t_translation() {
        let translator = mock_translator();
        let x: i32 = 150;
        let y: i32 = 250;
        let wasm_value: WasmUsize = ((y as WasmUsize) << POINT_Y_OFFSET) | (x as WasmUsize);

        let point = lvgl::lv_point_precise_t::translate_from(wasm_value, translator).unwrap();
        assert_eq!(point.x, x);
        assert_eq!(point.y, y);

        let result = point.translate_into(translator).unwrap();
        assert_eq!(result, wasm_value);
    }

    #[test]
    fn test_negative_point_translation() {
        let translator = mock_translator();
        let x: i32 = -50;
        let y: i32 = -100;
        let wasm_value: WasmUsize =
            ((y as WasmUsize) << POINT_Y_OFFSET) | ((x as WasmUsize) & POINT_MASK);

        let point = lvgl::lv_point_t::translate_from(wasm_value, translator).unwrap();
        let result = point.translate_into(translator).unwrap();
        assert_eq!(result, wasm_value);
    }
}
