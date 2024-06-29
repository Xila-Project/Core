mod Error;
pub use Error::*;

#[derive(Clone, Copy)]
pub struct Point_type {
    pub X: i16,
    pub Y: i16,
}

impl Point_type {
    pub fn New(X: i16, Y: i16) -> Self {
        Self { X, Y }
    }
}

pub struct Area_type {
    pub Position: Point_type,
    pub Size: Point_type,
}

impl Area_type {
    pub fn New(Position: Point_type, Size: Point_type) -> Self {
        Self { Position, Size }
    }
}

#[derive(Clone, Copy)]
pub struct Color_type {
    pub Red: u8,
    pub Green: u8,
    pub Blue: u8,
}

pub struct Refresh_area_type<const Buffer_size: usize> {
    pub Area: Area_type,
    pub Buffer: [Color_type; Buffer_size],
}

pub trait Screen_traits<const Buffer_size: usize> {
    fn Update(&mut self, Refresh_area: &Refresh_area_type<Buffer_size>);
    fn Get_resolution(&self) -> Result_type<Point_type>;
}

#[derive(Clone, Copy)]
pub enum Touch_type {
    Pressed,
    Released,
}

pub trait Input_traits {
    fn Get_latest_input(&self) -> (Point_type, Touch_type);
}
