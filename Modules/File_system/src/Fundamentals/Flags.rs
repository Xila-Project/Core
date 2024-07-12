use super::Permission_type;

#[inline]
fn Set_bit(Data: &mut u32, Position: u8, Value: bool) {
    if Value {
        *Data |= 1 << Position;
    } else {
        *Data &= !(1 << Position);
    }
}

#[inline]
fn Get_bit(Data: &u32, Bit: u8) -> bool {
    (Data & (1 << Bit)) != 0
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Mode_type(u32);

impl Mode_type {
    const Read_bit: u8 = 0;
    const Write_bit: u8 = 1;

    pub fn Read_only() -> Self {
        Self(1 << Self::Read_bit)
    }

    pub fn Write_only() -> Self {
        Self(1 << Self::Write_bit)
    }

    pub fn Read_write() -> Self {
        Self((1 << Self::Read_bit) | (1 << Self::Write_bit))
    }

    pub fn Get_read(&self) -> bool {
        Get_bit(&self.0, Self::Read_bit)
    }

    pub fn Get_write(&self) -> bool {
        Get_bit(&self.0, Self::Write_bit)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Status_type(u32);

impl Default for Status_type {
    fn default() -> Self {
        Self(0).Set_append(false).Set_non_blocking(false)
    }
}

impl Status_type {
    const Append_bit: u8 = 0;
    const Non_blocking_bit: u8 = 1;

    pub fn Set_non_blocking(mut self, Value: bool) -> Self {
        Set_bit(&mut self.0, Self::Non_blocking_bit, Value);
        self
    }

    pub fn Get_non_blocking(&self) -> bool {
        Get_bit(&self.0, Self::Non_blocking_bit)
    }

    pub fn Set_append(mut self, Value: bool) -> Self {
        Set_bit(&mut self.0, Self::Append_bit, Value);
        self
    }

    pub fn Get_append(&self) -> bool {
        Get_bit(&self.0, Self::Append_bit)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Flags_type(u32);

impl Flags_type {
    pub fn New(Mode: Mode_type, Status: Option<Status_type>) -> Self {
        Self(Mode.0 << 4 | Status.unwrap_or_default().0)
    }

    pub fn Get_mode(&self) -> Mode_type {
        Mode_type(self.0 >> 4)
    }

    pub fn Get_status(&self) -> Status_type {
        Status_type(self.0 & 0b00001111)
    }

    pub fn Set_status(&mut self, Status: Status_type) {
        self.0 = (self.0 & 0b11110000) | Status.0;
    }

    pub fn Is_permission_granted(&self, Permission: &Permission_type) -> bool {
        let Mode = self.Get_mode();

        (Permission.Get_read() && Mode.Get_read()) // Read permission
            || (Permission.Get_write() && (Mode.Get_write() || self.Get_status().Get_append()))
        // Write permission
    }
}

impl From<Mode_type> for Flags_type {
    fn from(Mode: Mode_type) -> Self {
        Self::New(Mode, None)
    }
}

impl From<Flags_type> for u32 {
    fn from(Flags: Flags_type) -> Self {
        Flags.0
    }
}
