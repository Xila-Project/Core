use embedded_graphics::geometry::Point;

use super::lvgl;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point_type {
    X: i16,
    Y: i16,
}

impl Point_type {
    pub const fn New(X: i16, Y: i16) -> Self {
        Self { X, Y }
    }

    pub const fn Get_x(&self) -> i16 {
        self.X
    }

    pub const fn Get_y(&self) -> i16 {
        self.Y
    }

    pub fn Split(self) -> (i16, i16) {
        (self.X, self.Y)
    }

    pub fn Set_x(mut self, Value: i16) -> Self {
        self.X = Value;
        self
    }

    pub fn Set_y(mut self, Value: i16) -> Self {
        self.Y = Value;
        self
    }

    pub fn Set(mut self, X: i16, Y: i16) -> Self {
        self.X = X;
        self.Y = Y;
        self
    }

    pub fn Get_distance(&self, Other: Point_type) -> f32 {
        let X = (self.X - Other.X) as f32;
        let Y = (self.Y - Other.Y) as f32;
        (X * X + Y * Y).sqrt()
    }
}

impl From<Point_type> for lvgl::lv_point_t {
    fn from(Point: Point_type) -> Self {
        Self {
            x: Point.X,
            y: Point.Y,
        }
    }
}

impl From<&Point> for Point_type {
    fn from(Point: &Point) -> Self {
        Self::New(Point.x as i16, Point.y as i16)
    }
}

impl From<&Point_type> for Point {
    fn from(Point: &Point_type) -> Self {
        Self::new(Point.X as i32, Point.Y as i32)
    }
}
