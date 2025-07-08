use crate::{Point_type, LVGL};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area_type(Point_type, Point_type);

impl Area_type {
    pub fn new(point_1: Point_type, Point_2: Point_type) -> Self {
        Self(point_1, Point_2)
    }

    pub fn get_point_1(&self) -> Point_type {
        self.0
    }

    pub fn get_point_2(&self) -> Point_type {
        self.1
    }

    pub fn get_width(&self) -> u16 {
        (self.1.get_x() - self.0.get_x()).unsigned_abs() + 1
    }

    pub fn get_height(&self) -> u16 {
        (self.1.get_y() - self.0.get_y()).unsigned_abs() + 1
    }

    pub fn Set_point_1(mut self, Value: Point_type) -> Self {
        self.0 = Value;
        self
    }

    pub fn Set_point_2(mut self, Value: Point_type) -> Self {
        self.1 = Value;
        self
    }
}

impl From<LVGL::lv_area_t> for Area_type {
    fn from(value: LVGL::lv_area_t) -> Self {
        Self::new(
            Point_type::new(value.x1 as i16, value.y1 as i16),
            Point_type::new(value.x2 as i16, value.y2 as i16),
        )
    }
}

impl AsRef<[u8]> for Area_type {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
        }
    }
}

impl AsRef<Area_type> for [u8; core::mem::size_of::<Area_type>()] {
    fn as_ref(&self) -> &Area_type {
        unsafe { &*(self as *const _ as *const Area_type) }
    }
}
