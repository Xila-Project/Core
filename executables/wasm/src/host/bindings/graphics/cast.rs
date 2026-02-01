use core::mem::transmute;

use xila::graphics::lvgl::{self, lv_style_value_t};

pub trait ToUsize {
    fn to_usize(&self) -> usize;
}

pub trait FromUsize: Sized {
    fn from_usize(value: usize) -> Self;
}

macro_rules! implicit_usize_cast {
    ($($t:ty),* $(,)?) => {
        $(
            impl ToUsize for $t {
                #[inline]
                fn to_usize(&self) -> usize {
                    *self as usize
                }
            }

            impl FromUsize for $t {
                #[inline]
                fn from_usize(value: usize) -> Self {
                    value as $t
                }
            }
        )*
    };
}

implicit_usize_cast!(u8, u16, u32, usize, i8, i16, i32, isize, f32);

#[cfg(target_pointer_width = "64")]
implicit_usize_cast!(u64, i64, f64);

impl FromUsize for bool {
    #[inline]
    fn from_usize(value: usize) -> Self {
        value != 0
    }
}

impl ToUsize for bool {
    #[inline]
    fn to_usize(&self) -> usize {
        if *self { 1 } else { 0 }
    }
}

impl<T> FromUsize for *mut T {
    #[inline]
    fn from_usize(value: usize) -> Self {
        value as *mut T
    }
}

impl<T> ToUsize for *mut T {
    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl<T> FromUsize for *const T {
    #[inline]
    fn from_usize(value: usize) -> Self {
        value as *const T
    }
}

impl<T> ToUsize for *const T {
    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl FromUsize for () {
    #[inline]
    fn from_usize(_value: usize) -> Self {
        ()
    }
}

impl ToUsize for () {
    #[inline]
    fn to_usize(&self) -> usize {
        0
    }
}

impl FromUsize for lvgl::lv_color_t {
    #[inline]
    fn from_usize(value: usize) -> Self {
        lvgl::lv_color_t {
            blue: value as u8,
            green: (value >> 8) as u8,
            red: (value >> 16) as u8,
        }
    }
}

impl ToUsize for lvgl::lv_color_t {
    #[inline]
    fn to_usize(&self) -> usize {
        (self.red as usize) << 16 | (self.green as usize) << 8 | (self.blue as usize)
    }
}

impl FromUsize for lvgl::lv_color32_t {
    #[inline]
    fn from_usize(value: usize) -> Self {
        lvgl::lv_color32_t {
            blue: value as u8,
            green: (value >> 8) as u8,
            red: (value >> 16) as u8,
            alpha: (value >> 24) as u8,
        }
    }
}

impl ToUsize for lvgl::lv_color32_t {
    #[inline]
    fn to_usize(&self) -> usize {
        (self.alpha as usize) << 24
            | (self.red as usize) << 16
            | (self.green as usize) << 8
            | (self.blue as usize)
    }
}

impl FromUsize for lvgl::lv_color16_t {
    #[inline]
    fn from_usize(value: usize) -> Self {
        unsafe { transmute::<u16, lvgl::lv_color16_t>(value as u16) }
    }
}

impl ToUsize for lvgl::lv_color16_t {
    #[inline]
    fn to_usize(&self) -> usize {
        unsafe { transmute::<lvgl::lv_color16_t, u16>(*self) as usize }
    }
}

impl FromUsize for lvgl::lv_style_value_t {
    #[inline]
    fn from_usize(value: usize) -> Self {
        let value = value as *mut lv_style_value_t;
        unsafe { *value }
    }
}

impl FromUsize for lvgl::lv_point_t {
    #[inline]
    fn from_usize(value: usize) -> Self {
        #[cfg(target_pointer_width = "32")]
        {
            lvgl::lv_point_t {
                x: (value & 0xFFFF) as i32,
                y: ((value >> 16) & 0xFFFF) as i32,
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            lvgl::lv_point_t {
                x: (value & 0xFFFFFFFF) as i32,
                y: ((value >> 32) & 0xFFFFFFFF) as i32,
            }
        }
    }
}

impl ToUsize for lvgl::lv_point_t {
    #[inline]
    fn to_usize(&self) -> usize {
        #[cfg(target_pointer_width = "32")]
        {
            (self.y as usize) << 16 | (self.x as usize)
        }
        #[cfg(target_pointer_width = "64")]
        {
            (self.y as usize) << 32 | (self.x as usize)
        }
    }
}

impl FromUsize for lvgl::lv_point_precise_t {
    #[inline]
    fn from_usize(value: usize) -> Self {
        #[cfg(target_pointer_width = "32")]
        {
            lvgl::lv_point_precise_t {
                x: (value & 0xFFFF) as i32,
                y: ((value >> 16) & 0xFFFF) as i32,
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            lvgl::lv_point_precise_t {
                x: (value & 0xFFFFFFFF) as i32,
                y: ((value >> 32) & 0xFFFFFFFF) as i32,
            }
        }
    }
}

impl ToUsize for lvgl::lv_point_precise_t {
    #[inline]
    fn to_usize(&self) -> usize {
        #[cfg(target_pointer_width = "32")]
        {
            (self.y as usize) << 16 | (self.x as usize)
        }
        #[cfg(target_pointer_width = "64")]
        {
            (self.y as usize) << 32 | (self.x as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xila::graphics::lvgl;

    #[test]
    fn test_u8_roundtrip() {
        let val: u8 = 42;
        assert_eq!(u8::from_usize(val.to_usize()), val);
    }

    #[test]
    fn test_u16_roundtrip() {
        let val: u16 = 1234;
        assert_eq!(u16::from_usize(val.to_usize()), val);
    }

    #[test]
    fn test_u32_roundtrip() {
        let val: u32 = 123456;
        assert_eq!(u32::from_usize(val.to_usize()), val);
    }

    #[test]
    fn test_bool_roundtrip() {
        assert_eq!(bool::from_usize(true.to_usize()), true);
        assert_eq!(bool::from_usize(false.to_usize()), false);
        assert_eq!(bool::from_usize(0), false);
        assert_eq!(bool::from_usize(1), true);
        assert_eq!(bool::from_usize(42), true);
    }

    #[test]
    fn test_pointer_roundtrip() {
        let val: *mut i32 = 0x1234 as *mut i32;
        assert_eq!(<*mut i32>::from_usize(val.to_usize()), val);

        let val: *const i32 = 0x5678 as *const i32;
        assert_eq!(<*const i32>::from_usize(val.to_usize()), val);
    }

    #[test]
    fn test_unit_roundtrip() {
        let val = ();
        assert_eq!(<()>::from_usize(val.to_usize()), val);
        assert_eq!(val.to_usize(), 0);
    }

    #[test]
    fn test_lv_color_t_roundtrip() {
        let color = lvgl::lv_color_t {
            red: 0xFF,
            green: 0x80,
            blue: 0x40,
        };
        let roundtrip = lvgl::lv_color_t::from_usize(color.to_usize());
        assert_eq!(roundtrip.red, color.red);
        assert_eq!(roundtrip.green, color.green);
        assert_eq!(roundtrip.blue, color.blue);
    }

    #[test]
    fn test_lv_color32_t_roundtrip() {
        let color = lvgl::lv_color32_t {
            red: 0xFF,
            green: 0x80,
            blue: 0x40,
            alpha: 0xCC,
        };
        let roundtrip = lvgl::lv_color32_t::from_usize(color.to_usize());
        assert_eq!(roundtrip.red, color.red);
        assert_eq!(roundtrip.green, color.green);
        assert_eq!(roundtrip.blue, color.blue);
        assert_eq!(roundtrip.alpha, color.alpha);
    }

    #[test]
    fn test_lv_point_t_roundtrip() {
        let point = lvgl::lv_point_t { x: 100, y: 200 };
        let roundtrip = lvgl::lv_point_t::from_usize(point.to_usize());
        assert_eq!(roundtrip.x, point.x);
        assert_eq!(roundtrip.y, point.y);
    }

    #[test]
    fn test_lv_point_t_negative() {
        let point = lvgl::lv_point_t { x: -50, y: -100 };
        let roundtrip = lvgl::lv_point_t::from_usize(point.to_usize());
        assert_eq!(roundtrip.x, point.x);
        assert_eq!(roundtrip.y, point.y);
    }

    #[test]
    fn test_lv_point_t_zero() {
        let point = lvgl::lv_point_t { x: 0, y: 0 };
        let roundtrip = lvgl::lv_point_t::from_usize(point.to_usize());
        assert_eq!(roundtrip.x, point.x);
        assert_eq!(roundtrip.y, point.y);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_u64_roundtrip() {
        let val: u64 = 0x123456789ABCDEF0;
        assert_eq!(u64::from_usize(val.to_usize()), val);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_i64_roundtrip() {
        let val: i64 = -123456789;
        assert_eq!(i64::from_usize(val.to_usize()), val);
    }
}
