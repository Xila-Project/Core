use alloc::slice;
use shared::flags;

flags! {
    /// The kinds of control commands.
    pub enum ControlDirectionFlags: u8 {
        /// Read data from the device.
        Read,
        /// Write data to the device.
        Write,
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlCommand(usize);

impl ControlCommand {
    const NUMBER_SIZE: usize = 8;
    const KIND_SIZE: usize = 8;
    const SIZE_SIZE: usize = 14;
    const DIRECTION_SIZE: usize = 2;

    const NUMBER_SHIFT: usize = 0;
    const KIND_SHIFT: usize = Self::NUMBER_SHIFT + Self::NUMBER_SIZE; // 8
    const SIZE_SHIFT: usize = Self::KIND_SHIFT + Self::KIND_SIZE; // 16
    const DIRECTION_SHIFT: usize = Self::SIZE_SHIFT + Self::SIZE_SIZE; // 30

    pub const fn new<A>(direction: ControlDirectionFlags, kind: u8, number: u8) -> Self {
        Self::new_with_size(direction, size_of::<A>(), kind, number)
    }

    pub const fn new_with_size(
        direction: ControlDirectionFlags,
        size: usize,
        kind: u8,
        number: u8,
    ) -> Self {
        let direction = direction.bits() as usize;
        let kind = kind as usize;
        let number = number as usize;

        Self(
            (direction << Self::DIRECTION_SHIFT)
                | (size << Self::SIZE_SHIFT)
                | (kind << Self::KIND_SHIFT)
                | (number << Self::NUMBER_SHIFT),
        )
    }

    pub const fn get_direction(&self) -> ControlDirectionFlags {
        let direction = (self.0 >> Self::DIRECTION_SHIFT) & ((1 << Self::DIRECTION_SIZE) - 1);
        ControlDirectionFlags(direction as u8)
    }

    pub const fn get_size(&self) -> usize {
        (self.0 >> Self::SIZE_SHIFT) & ((1 << Self::SIZE_SIZE) - 1)
    }

    pub const fn get_kind(&self) -> u8 {
        ((self.0 >> Self::KIND_SHIFT) & ((1 << Self::KIND_SIZE) - 1)) as u8
    }

    pub const fn get_number(&self) -> u8 {
        ((self.0 >> Self::NUMBER_SHIFT) & ((1 << Self::NUMBER_SIZE) - 1)) as u8
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct ControlArgument([u8]);

impl<'a, T> From<&'a mut T> for &'a mut ControlArgument {
    fn from(argument: &'a mut T) -> Self {
        ControlArgument::from(argument)
    }
}

impl ControlArgument {
    pub fn from<T>(argument: &mut T) -> &mut Self {
        unsafe {
            let slice = slice::from_raw_parts_mut(argument as *mut T as *mut u8, size_of::<T>());
            &mut *(slice as *mut [u8] as *mut Self)
        }
    }

    pub fn cast<T>(&mut self) -> Option<&mut T> {
        if size_of::<T>() > self.0.len() {
            return None;
        }

        let argument = self.0.as_mut_ptr();

        if argument.is_null() {
            return None;
        }

        // check alignment
        if argument.align_offset(align_of::<T>()) != 0 {
            return None;
        }

        Some(unsafe { &mut *(argument as *mut T) })
    }

    pub fn get_size(&self) -> usize {
        self.0.len()
    }

    pub fn get_alignment(&self) -> usize {
        self.0.as_ptr().align_offset(1)
    }

    pub fn as_mutable_bytes(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
