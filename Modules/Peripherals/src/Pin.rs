use core::mem::{size_of, transmute};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction_type {
    Input = 0,
    Output = 1,
}

impl TryFrom<u8> for Direction_type {
    type Error = ();

    fn try_from(Value: u8) -> Result<Self, Self::Error> {
        match Value {
            0 => Ok(Self::Input),
            1 => Ok(Self::Output),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Level_type {
    Low = 0,
    High = 1,
}

impl TryFrom<u8> for Level_type {
    type Error = ();

    fn try_from(Value: u8) -> Result<Self, Self::Error> {
        match Value {
            0 => Ok(Self::Low),
            1 => Ok(Self::High),
            _ => Err(()),
        }
    }
}

impl From<Level_type> for u8 {
    fn from(Value: Level_type) -> u8 {
        Value as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pull_type {
    None = 0,
    Up = 1,
    Down = 2,
    Up_down = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Pin_data_type {
    Level: Option<Level_type>,
    Direction: Option<Direction_type>,
    Pull: Option<Pull_type>,
}

impl Pin_data_type {
    pub const fn New(
        Level: Option<Level_type>,
        Direction: Option<Direction_type>,
        Pull: Option<Pull_type>,
    ) -> Self {
        Self {
            Level,
            Direction,
            Pull,
        }
    }

    pub const fn Get_level(&self) -> Option<Level_type> {
        self.Level
    }

    pub const fn Get_direction(&self) -> Option<Direction_type> {
        self.Direction
    }

    pub const fn Get_pull(&self) -> Option<Pull_type> {
        self.Pull
    }

    pub fn Set_level(&mut self, Level: Option<Level_type>) {
        self.Level = Level;
    }

    pub fn Set_direction(&mut self, Direction: Option<Direction_type>) {
        self.Direction = Direction;
    }

    pub fn Set_pull(&mut self, Pull: Option<Pull_type>) {
        self.Pull = Pull;
    }

    pub fn Set(
        &mut self,
        Level: Option<Level_type>,
        Direction: Option<Direction_type>,
        Pull: Option<Pull_type>,
    ) {
        self.Level = Level;
        self.Direction = Direction;
        self.Pull = Pull;
    }
}

impl AsMut<[u8]> for Pin_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut _ as *mut u8, core::mem::size_of::<Self>())
        }
    }
}

impl TryFrom<&[u8]> for &Pin_data_type {
    type Error = ();

    fn try_from(Value: &[u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Pin_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Pin_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*const u8, Self>(Value.as_ptr()) })
    }
}

impl TryFrom<&mut [u8]> for &mut Pin_data_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Pin_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Pin_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(Value.as_mut_ptr()) })
    }
}
