use std::fmt::Debug;

use super::Permission_type;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Mode_type(u8);

impl Debug for Mode_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Mode_type")
            .field("Read", &self.Get_read())
            .field("Write", &self.Get_write())
            .finish()
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Open_type(u8);

impl Open_type {
    pub const Create: u8 = 1 << 0;
    pub const Create_only: u8 = 1 << 1;
    pub const Truncate: u8 = 1 << 2;
    pub const Directory: u8 = 1 << 3;

    pub const Size: u8 = 4;

    pub const None: Self = Self::New(false, false, false, false);

    pub const fn New(Create: bool, Create_only: bool, Truncate: bool, Directory: bool) -> Self {
        Self(0)
            .Set_create(Create)
            .Set_create_only(Create_only)
            .Set_truncate(Truncate)
            .Set_directory(Directory)
    }

    pub const fn Get_bit(&self, Mask: u8) -> bool {
        self.0 & Mask != 0
    }

    pub const fn Set_bit(mut self, Mask: u8, Value: bool) -> Self {
        if Value {
            self.0 |= Mask;
        } else {
            self.0 &= !Mask;
        }
        self
    }

    pub const fn Get_create(&self) -> bool {
        self.Get_bit(Self::Create)
    }

    pub const fn Set_create(self, Value: bool) -> Self {
        self.Set_bit(Self::Create, Value)
    }

    pub const fn Get_create_only(&self) -> bool {
        self.Get_bit(Self::Create_only)
    }

    pub const fn Set_create_only(self, Value: bool) -> Self {
        self.Set_bit(Self::Create_only, Value)
    }

    pub const fn Get_truncate(&self) -> bool {
        self.Get_bit(Self::Truncate)
    }

    pub const fn Set_truncate(self, Value: bool) -> Self {
        self.Set_bit(Self::Truncate, Value)
    }

    pub const fn Get_directory(&self) -> bool {
        self.Get_bit(Self::Directory)
    }

    pub const fn Set_directory(self, Value: bool) -> Self {
        self.Set_bit(Self::Directory, Value)
    }
}

impl Default for Open_type {
    fn default() -> Self {
        Self::None
    }
}

impl Debug for Open_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Open_type")
            .field("Create", &self.Get_create())
            .field("Create_only", &self.Get_create_only())
            .field("Truncate", &self.Get_truncate())
            .field("Directory", &self.Get_directory())
            .finish()
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Status_type(u8);

impl Debug for Status_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Status_type")
            .field("Append", &self.Get_append())
            .field("Non_blocking", &self.Get_non_blocking())
            .finish()
    }
}

impl Default for Status_type {
    fn default() -> Self {
        Self(0).Set_append(false).Set_non_blocking(false)
    }
}

impl Status_type {
    pub const Append_bit: u8 = 1 << 0;
    pub const Non_blocking_bit: u8 = 1 << 1;

    fn Set_bit(&mut self, Mask: u8, Value: bool) {
        if Value {
            self.0 |= Mask;
        } else {
            self.0 &= !Mask;
        }
    }

    fn Get_bit(&self, Mask: u8) -> bool {
        self.0 & Mask != 0
    }

    pub fn Set_non_blocking(mut self, Value: bool) -> Self {
        self.Set_bit(Self::Non_blocking_bit, Value);
        self
    }

    pub fn Get_non_blocking(&self) -> bool {
        self.Get_bit(Self::Non_blocking_bit)
    }

    pub fn Set_append(mut self, Value: bool) -> Self {
        self.Set_bit(Self::Append_bit, Value);
        self
    }

    pub fn Get_append(&self) -> bool {
        self.Get_bit(Self::Append_bit)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Flags_type(u16);

impl Flags_type {
    pub fn New(Mode: Mode_type, Status: Option<Status_type>) -> Self {
        Self((Mode.0 as u16) << 4 | (Status.unwrap_or_default().0 as u16))
    }

    pub fn Get_mode(&self) -> Mode_type {
        Mode_type((self.0 >> 4) as u8)
    }

    pub fn Get_status(&self) -> Status_type {
        Status_type((self.0 & 0b00001111) as u8)
    }

    pub fn Set_status(&mut self, Status: Status_type) {
        self.0 = (self.0 & 0b11110000) | (Status.0 as u16);
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

impl From<Flags_type> for u16 {
    fn from(Flags: Flags_type) -> Self {
        Flags.0
    }
}
