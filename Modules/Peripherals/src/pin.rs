use core::mem::{size_of, transmute};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Input = 0,
    Output = 1,
}

impl TryFrom<u8> for Direction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Input),
            1 => Ok(Self::Output),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Level {
    Low = 0,
    High = 1,
}

impl TryFrom<u8> for Level {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Low),
            1 => Ok(Self::High),
            _ => Err(()),
        }
    }
}

impl From<Level> for u8 {
    fn from(value: Level) -> u8 {
        value as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pull {
    None = 0,
    Up = 1,
    Down = 2,
    UpDown = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct PinData {
    level: Option<Level>,
    direction: Option<Direction>,
    pull: Option<Pull>,
}

impl PinData {
    pub const fn new(
        level: Option<Level>,
        direction: Option<Direction>,
        pull: Option<Pull>,
    ) -> Self {
        Self {
            level,
            direction,
            pull,
        }
    }

    pub const fn get_level(&self) -> Option<Level> {
        self.level
    }

    pub const fn get_direction(&self) -> Option<Direction> {
        self.direction
    }

    pub const fn get_pull(&self) -> Option<Pull> {
        self.pull
    }

    pub fn set_level(&mut self, level: Option<Level>) {
        self.level = level;
    }

    pub fn set_direction(&mut self, direction: Option<Direction>) {
        self.direction = direction;
    }

    pub fn set_pull(&mut self, pull: Option<Pull>) {
        self.pull = pull;
    }

    pub fn set(&mut self, level: Option<Level>, direction: Option<Direction>, pull: Option<Pull>) {
        self.level = level;
        self.direction = direction;
        self.pull = pull;
    }
}

impl AsMut<[u8]> for PinData {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut _ as *mut u8, core::mem::size_of::<Self>())
        }
    }
}

impl TryFrom<&[u8]> for &PinData {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<PinData>() {
            return Err(());
        }
        if !(value.as_ptr() as usize).is_multiple_of(core::mem::align_of::<PinData>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*const u8, Self>(value.as_ptr()) })
    }
}

impl TryFrom<&mut [u8]> for &mut PinData {
    type Error = ();

    fn try_from(value: &mut [u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<PinData>() {
            return Err(());
        }
        if !(value.as_ptr() as usize).is_multiple_of(core::mem::align_of::<PinData>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(value.as_mut_ptr()) })
    }
}
