use super::lvgl;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Point_type {
    x: i16,
    y: i16,
}

impl Point_type {
    pub const fn new(X: i16, Y: i16) -> Self {
        Self { x: X, y: Y }
    }

    pub const fn get_x(&self) -> i16 {
        self.x
    }

    pub const fn get_y(&self) -> i16 {
        self.y
    }

    pub fn Split(self) -> (i16, i16) {
        (self.x, self.y)
    }

    pub fn Set_x(mut self, Value: i16) -> Self {
        self.x = Value;
        self
    }

    pub fn Set_y(mut self, Value: i16) -> Self {
        self.y = Value;
        self
    }

    pub fn Set(mut self, X: i16, Y: i16) -> Self {
        self.x = X;
        self.y = Y;
        self
    }

    pub fn get_distance(&self, Other: Point_type) -> f32 {
        let x = (self.x - Other.x) as f32;
        let y = (self.y - Other.y) as f32;
        (x * x + y * y).sqrt()
    }
}

impl From<(i16, i16)> for Point_type {
    fn from((x, y): (i16, i16)) -> Self {
        Self::new(x, y)
    }
}

impl From<Point_type> for (i16, i16) {
    fn from(point: Point_type) -> Self {
        point.Split()
    }
}

impl From<Point_type> for lvgl::lv_point_t {
    fn from(point: Point_type) -> Self {
        Self {
            x: point.x as i32,
            y: point.y as i32,
        }
    }
}
