use super::Point_type;

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

    pub fn Get_width(&self) -> i16 {
        self.1.Get_x() - self.0.Get_x()
    }

    pub fn Get_height(&self) -> i16 {
        self.1.Get_y() - self.0.Get_y()
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
