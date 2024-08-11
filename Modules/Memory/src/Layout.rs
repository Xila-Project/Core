#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout_type {
    Size: usize,
    Align: u8,
}

impl Layout_type {
    pub fn New(Size: usize, Align: u8) -> Self {
        Layout_type { Size, Align }
    }

    pub const fn Get_size(&self) -> usize {
        self.Size
    }

    pub const fn Get_alignment(&self) -> u8 {
        self.Align
    }

    pub fn Set_size(&mut self, Size: usize) -> &mut Self {
        self.Size = Size;
        self
    }

    pub fn Set_alignment(&mut self, Align: u8) -> &mut Self {
        self.Align = Align;
        self
    }
}
