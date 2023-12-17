pub struct Coordinates_type {
    pub X : i16,
    pub Y : i16,
}

pub struct Area_type {
    pub Position : Coordinates_type,
    pub Size : Coordinates_type,
}

#[derive(Clone, Copy)]
pub struct Color_type {
    pub Red : u8,
    pub Green : u8,
    pub Blue : u8,
}

pub struct Refresh_area_type<const Buffer_size : usize> {
    pub Area : Area_type,
    pub Buffer : [Color_type; Buffer_size],
}

pub trait Screen_traits<const Buffer_size : usize> {
    fn Update(&mut self, Refresh_area : &Refresh_area_type<Buffer_size>);
    fn Get_resolution(&self) -> Result<Coordinates_type, ()>;
}
