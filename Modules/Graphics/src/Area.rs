use crate::{lvgl, Point_type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area_type(Point_type, Point_type);

impl Area_type {
    pub fn New(Point_1: Point_type, Point_2: Point_type) -> Self {
        Self(Point_1, Point_2)
    }

    pub fn Get_point_1(&self) -> Point_type {
        self.0
    }

    pub fn Get_point_2(&self) -> Point_type {
        self.1
    }

    pub fn Get_width(&self) -> u16 {
        (self.1.Get_x() - self.0.Get_x()).abs() as u16
    }

    pub fn Get_height(&self) -> u16 {
        (self.1.Get_y() - self.0.Get_y()).abs() as u16
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

impl From<lvgl::lv_area_t> for Area_type {
    fn from(Value: lvgl::lv_area_t) -> Self {
        Self::New(
            Point_type::New(Value.x1 as i16, Value.y1 as i16),
            Point_type::New(Value.x2 as i16, Value.y2 as i16),
        )
    }
}

impl AsRef<[u8]> for Area_type {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
        }
    }
}

impl AsRef<Area_type> for [u8; std::mem::size_of::<Area_type>()] {
    fn as_ref(&self) -> &Area_type {
        unsafe { &*(self as *const _ as *const Area_type) }
    }
}
