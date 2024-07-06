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
