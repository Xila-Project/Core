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
    fn from(value: Level_type) -> u8 {
        value as u8
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
    level: Option<Level_type>,
    direction: Option<Direction_type>,
    pull: Option<Pull_type>,
}

impl Pin_data_type {
    pub const fn new(
        level: Option<Level_type>,
        direction: Option<Direction_type>,
        pull: Option<Pull_type>,
    ) -> Self {
        Self {
            level,
            direction,
            pull,
        }
    }

    pub const fn Get_level(&self) -> Option<Level_type> {
        self.level
    }

    pub const fn Get_direction(&self) -> Option<Direction_type> {
        self.direction
    }

    pub const fn Get_pull(&self) -> Option<Pull_type> {
        self.pull
    }

    pub fn Set_level(&mut self, Level: Option<Level_type>) {
        self.level = Level;
    }

    pub fn Set_direction(&mut self, Direction: Option<Direction_type>) {
        self.direction = Direction;
    }

    pub fn Set_pull(&mut self, Pull: Option<Pull_type>) {
        self.pull = Pull;
    }

    pub fn Set(
        &mut self,
        level: Option<Level_type>,
        direction: Option<Direction_type>,
        pull: Option<Pull_type>,
    ) {
        self.level = level;
        self.direction = direction;
        self.pull = pull;
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
        if !(Value.as_ptr() as usize).is_multiple_of(core::mem::align_of::<Pin_data_type>()) {
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
        if !(Value.as_ptr() as usize).is_multiple_of(core::mem::align_of::<Pin_data_type>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(Value.as_mut_ptr()) })
    }
}
