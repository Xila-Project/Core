use core::fmt::Debug;

use super::Permission_type;

/// The mode of a file.
///
/// The mode is stored in a 8-bit integer, with the following layout:
///
/// | Read | Write |
/// |------|-------|
/// | 0    | 1     |
///
/// # Example
///
/// ```rust
/// use file_system::Mode_type;
///
/// let Mode = Mode_type::New(true, false);
///
/// assert_eq!(Mode.get_read(), true);
/// assert_eq!(Mode.get_write(), false);
///
/// let Mode = Mode_type::New(false, true);
///
/// assert_eq!(Mode.get_read(), false);
/// assert_eq!(Mode.get_write(), true);
///
/// let Mode = Mode_type::New(true, true);
///
/// assert_eq!(Mode.get_read(), true);
/// assert_eq!(Mode.get_write(), true);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Mode_type(u8);

impl Mode_type {
    pub const READ_BIT: u8 = 1 << 0;
    pub const WRITE_BIT: u8 = 1 << 1;

    pub const SIZE: u8 = 2;

    pub const READ_ONLY: Self = Self::New(true, false);
    pub const WRITE_ONLY: Self = Self::New(false, true);
    pub const READ_WRITE: Self = Self::New(true, true);

    pub const fn New(Read: bool, Write: bool) -> Self {
        Self(0).Set_read(Read).Set_write(Write)
    }

    pub const fn Set_bit(mut self, Mask: u8, Value: bool) -> Self {
        if Value {
            self.0 |= Mask;
        } else {
            self.0 &= !Mask;
        }
        self
    }

    pub const fn Set_read(self, Value: bool) -> Self {
        self.Set_bit(Self::READ_BIT, Value)
    }

    pub const fn Set_write(self, Value: bool) -> Self {
        self.Set_bit(Self::WRITE_BIT, Value)
    }

    pub const fn get_bit(&self, Mask: u8) -> bool {
        self.0 & Mask != 0
    }

    pub const fn get_read(&self) -> bool {
        self.get_bit(Self::READ_BIT)
    }

    pub const fn get_write(&self) -> bool {
        self.get_bit(Self::WRITE_BIT)
    }

    pub const fn From_u8(Value: u8) -> Self {
        Self(Value)
    }

    pub const fn As_u8(&self) -> u8 {
        self.0
    }
}

impl Debug for Mode_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Mode_type")
            .field("Read", &self.get_read())
            .field("Write", &self.get_write())
            .finish()
    }
}

/// The type of opening a file.
///
/// The type is stored in a 8-bit integer, with the following layout:
///
/// | Create | Create exclusive | Truncate | Directory |
/// |--------|------------------|----------|-----------|
/// | 0      | 1                | 2        | 3         |
///
/// # Example
///
/// ```rust
/// use file_system::Open_type;
///
/// let Open = Open_type::New(true, true, false);
///
/// assert_eq!(Open.get_create(), true);
/// assert_eq!(Open.get_exclusive(), true);
/// assert_eq!(Open.get_truncate(), false);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Open_type(u8);

impl Open_type {
    pub const CREATE_MASK: u8 = 1 << 0;
    pub const EXCLUSIVE_MASK: u8 = 1 << 1;
    pub const TRUNCATE_MASK: u8 = 1 << 2;

    pub const SIZE: u8 = 3;

    pub const NONE: Self = Self::New(false, false, false);

    pub const CREATE: Self = Self::New(true, false, false);
    pub const CREATE_ONLY: Self = Self::New(true, true, false);
    pub const TRUNCATE: Self = Self::New(false, false, true);

    pub const fn New(Create: bool, Create_only: bool, Truncate: bool) -> Self {
        Self(0)
            .Set_create(Create)
            .Set_exclusive(Create_only)
            .Set_truncate(Truncate)
    }

    pub const fn get_bit(&self, Mask: u8) -> bool {
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

    pub const fn get_create(&self) -> bool {
        self.get_bit(Self::CREATE_MASK)
    }

    pub const fn Set_create(self, Value: bool) -> Self {
        self.Set_bit(Self::CREATE_MASK, Value)
    }

    pub const fn get_exclusive(&self) -> bool {
        self.get_bit(Self::EXCLUSIVE_MASK)
    }

    pub const fn Set_exclusive(self, Value: bool) -> Self {
        self.Set_bit(Self::EXCLUSIVE_MASK, Value)
    }

    pub const fn get_truncate(&self) -> bool {
        self.get_bit(Self::TRUNCATE_MASK)
    }

    pub const fn Set_truncate(self, Value: bool) -> Self {
        self.Set_bit(Self::TRUNCATE_MASK, Value)
    }

    pub const fn From_u8(Value: u8) -> Self {
        Self(Value)
    }
}

impl Default for Open_type {
    fn default() -> Self {
        Self::NONE
    }
}

impl Debug for Open_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Open_type")
            .field("Create", &self.get_create())
            .field("Create_only", &self.get_exclusive())
            .field("Truncate", &self.get_truncate())
            .finish()
    }
}

/// The status of a file.
///
/// The status is stored in a 8-bit integer, with the following layout:
///
/// | Append | Non-blocking | Synchronous | Synchronous data only |
///  -------------------------------------------------------------
/// | 0      | 1            | 2           | 3                     |
///
/// # Example
///
/// ```rust
/// use file_system::Status_type;
///
/// let Status = Status_type::New(true, false, true, false);
///
/// assert_eq!(Status.get_append(), true);
/// assert_eq!(Status.get_non_blocking(), false);
/// assert_eq!(Status.get_synchronous(), true);
/// assert_eq!(Status.get_synchronous_data_only(), false);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Status_type(u8);

impl Status_type {
    pub const APPEND_BIT: u8 = 1 << 0;
    pub const NON_BLOCKING_BIT: u8 = 1 << 1;
    pub const SYNCHRONOUS_BIT: u8 = 1 << 2;
    pub const SYNCHRONOUS_DATA_ONLY_BIT: u8 = 1 << 3;

    pub const SIZE: u8 = 4;

    pub const NON_BLOCKING: Self = Self::New(false, true, false, false);

    pub const NONE: Self = Self::New(false, false, false, false);

    pub const fn New(
        append: bool,
        non_blocking: bool,
        synchronous: bool,
        synchronous_data_only: bool,
    ) -> Self {
        Self(0)
            .Set_append(append)
            .Set_non_blocking(non_blocking)
            .Set_synchronous(synchronous)
            .Set_synchronous_data_only(synchronous_data_only)
    }

    const fn Set_bit(mut self, Mask: u8, Value: bool) -> Self {
        if Value {
            self.0 |= Mask;
        } else {
            self.0 &= !Mask;
        }
        self
    }

    const fn get_bit(&self, Mask: u8) -> bool {
        self.0 & Mask != 0
    }

    pub const fn Set_non_blocking(self, Value: bool) -> Self {
        self.Set_bit(Self::NON_BLOCKING_BIT, Value)
    }

    pub fn get_non_blocking(&self) -> bool {
        self.get_bit(Self::NON_BLOCKING_BIT)
    }

    pub const fn Set_append(self, Value: bool) -> Self {
        self.Set_bit(Self::APPEND_BIT, Value)
    }

    pub const fn get_append(&self) -> bool {
        self.get_bit(Self::APPEND_BIT)
    }

    pub const fn Set_synchronous(self, Value: bool) -> Self {
        self.Set_bit(Self::SYNCHRONOUS_BIT, Value)
    }

    pub const fn get_synchronous(&self) -> bool {
        self.get_bit(Self::SYNCHRONOUS_BIT)
    }

    pub const fn Set_synchronous_data_only(self, Value: bool) -> Self {
        self.Set_bit(Self::SYNCHRONOUS_DATA_ONLY_BIT, Value)
    }

    pub const fn get_synchronous_data_only(&self) -> bool {
        self.get_bit(Self::SYNCHRONOUS_DATA_ONLY_BIT)
    }

    pub const fn From_u8(Value: u8) -> Self {
        Self(Value)
    }
}

impl Debug for Status_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Status_type")
            .field("Append", &self.get_append())
            .field("Non_blocking", &self.get_non_blocking())
            .field("Synchronous", &self.get_bit(Self::SYNCHRONOUS_BIT))
            .field(
                "Synchronous_data_only",
                &self.get_bit(Self::SYNCHRONOUS_DATA_ONLY_BIT),
            )
            .finish()
    }
}

impl Default for Status_type {
    fn default() -> Self {
        Self::NONE
    }
}

/// All the flags that can be set for a file.
///
/// The flags are stored in a 16-bit integer, with the following layout:
///
/// | Mode | Open | Status |
/// |------|------|--------|
/// | 0-1  | 2-5  | 6-9    |
///
/// # Example
///
/// ```rust
/// use file_system::{Flags_type, Mode_type, Open_type, Status_type};
///     
/// let Flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), Some(Status_type::Non_blocking));
///
/// assert_eq!(Flags.get_mode(), Mode_type::Read_write);
/// assert_eq!(Flags.get_open(), Open_type::Create_only);
/// assert_eq!(Flags.get_status(), Status_type::Non_blocking);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Flags_type(u16);

impl Debug for Flags_type {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Flags_type")
            .field("Mode", &self.get_mode())
            .field("Open", &self.get_open())
            .field("Status", &self.get_status())
            .finish()
    }
}

impl Flags_type {
    const MODE_POSITION: u8 = 0;
    const OPEN_POSITION: u8 = Mode_type::SIZE;
    const STATUS_POSITION: u8 = Open_type::SIZE + Self::OPEN_POSITION;

    const OPEN_MASK: u16 = (1 << Open_type::SIZE) - 1;
    const STATUS_MASK: u16 = (1 << Status_type::SIZE) - 1;
    const MODE_MASK: u16 = (1 << Mode_type::SIZE) - 1;

    pub const fn New(
        mode: Mode_type,
        open: Option<Open_type>,
        status: Option<Status_type>,
    ) -> Self {
        let open = if let Some(Open) = open {
            Open
        } else {
            Open_type::NONE
        };
        let Status = if let Some(Status) = status {
            Status
        } else {
            Status_type::NONE
        };

        let mut Flags: u16 = 0;
        Flags |= (mode.0 as u16) << Self::MODE_POSITION;
        Flags |= (open.0 as u16) << Self::OPEN_POSITION;
        Flags |= (Status.0 as u16) << Self::STATUS_POSITION;
        Self(Flags)
    }

    pub const fn get_mode(&self) -> Mode_type {
        Mode_type(((self.0 >> Self::MODE_POSITION) & Self::MODE_MASK) as u8)
    }

    pub const fn get_open(&self) -> Open_type {
        Open_type(((self.0 >> Self::OPEN_POSITION) & Self::OPEN_MASK) as u8)
    }

    pub const fn get_status(&self) -> Status_type {
        Status_type(((self.0 >> Self::STATUS_POSITION) & Self::STATUS_MASK) as u8)
    }

    pub const fn Set_mode(mut self, Mode: Mode_type) -> Self {
        self.0 &= !(Self::MODE_MASK << Self::MODE_POSITION);
        self.0 |= (Mode.0 as u16) << Self::MODE_POSITION;
        self
    }

    pub const fn Set_open(mut self, Open: Open_type) -> Self {
        self.0 &= !(Self::OPEN_MASK << Self::OPEN_POSITION);
        self.0 |= (Open.0 as u16) << Self::OPEN_POSITION;
        self
    }

    pub const fn Set_status(mut self, Status: Status_type) -> Self {
        self.0 &= !(Self::STATUS_MASK << Self::STATUS_POSITION);
        self.0 |= (Status.0 as u16) << Self::STATUS_POSITION;
        self
    }

    pub fn is_permission_granted(&self, Permission: &Permission_type) -> bool {
        let mode = self.get_mode();

        (Permission.get_read() && mode.get_read()) // Read permission
            || (Permission.get_write() && (mode.get_write() || self.get_status().get_append()))
        // Write permission
    }
}

impl From<Mode_type> for Flags_type {
    fn from(mode: Mode_type) -> Self {
        Self::New(mode, None, None)
    }
}

impl From<Flags_type> for u16 {
    fn from(flags: Flags_type) -> Self {
        flags.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_type_new() {
        let Read_only = Mode_type::New(true, false);
        assert!(Read_only.get_read());
        assert!(!Read_only.get_write());

        let Write_only = Mode_type::New(false, true);
        assert!(!Write_only.get_read());
        assert!(Write_only.get_write());

        let Read_write = Mode_type::New(true, true);
        assert!(Read_write.get_read());
        assert!(Read_write.get_write());
    }

    #[test]
    fn test_mode_type_set_get() {
        let mut Mode = Mode_type(0);
        Mode = Mode.Set_read(true);
        assert!(Mode.get_read());
        assert!(!Mode.get_write());

        Mode = Mode.Set_write(true);
        assert!(Mode.get_read());
        assert!(Mode.get_write());

        Mode = Mode.Set_read(false);
        assert!(!Mode.get_read());
        assert!(Mode.get_write());
    }

    #[test]
    fn test_open_type_new() {
        let Open = Open_type::New(true, false, true);
        assert!(Open.get_create());
        assert!(!Open.get_exclusive());
        assert!(Open.get_truncate());
    }

    #[test]
    fn test_open_type_set_get() {
        let mut Open = Open_type(0);
        Open = Open.Set_create(true);
        assert!(Open.get_create());
        assert!(!Open.get_exclusive());

        Open = Open.Set_exclusive(true);
        assert!(Open.get_create());
        assert!(Open.get_exclusive());

        Open = Open.Set_truncate(true);
        assert!(Open.get_truncate());
    }

    #[test]
    fn test_status_type_new() {
        let Status = Status_type::New(true, false, true, false);
        assert!(Status.get_append());
        assert!(!Status.get_non_blocking());
        assert!(Status.get_synchronous());
        assert!(!Status.get_synchronous_data_only());
    }

    #[test]
    fn test_status_type_set_get() {
        let mut Status = Status_type(0);
        Status = Status.Set_append(true);
        assert!(Status.get_append());
        assert!(!Status.get_non_blocking());

        Status = Status.Set_non_blocking(true);
        assert!(Status.get_non_blocking());

        Status = Status.Set_synchronous(true);
        assert!(Status.get_synchronous());

        Status = Status.Set_synchronous_data_only(true);
        assert!(Status.get_synchronous_data_only());
    }

    #[test]
    fn test_flags_type_new() {
        let Mode = Mode_type::READ_WRITE;
        let Open = Open_type::New(true, false, true);
        let Status = Status_type::New(true, false, true, false);

        let Flags = Flags_type::New(Mode, Some(Open), Some(Status));
        assert_eq!(Flags.get_mode(), Mode);
        assert_eq!(Flags.get_open(), Open);
        assert_eq!(Flags.get_status(), Status);
    }

    #[test]
    fn test_flags_type_set_get() {
        let Flags = Flags_type::New(Mode_type::READ_ONLY, None, None);

        let New_mode = Mode_type::WRITE_ONLY;
        let Flags = Flags.Set_mode(New_mode);
        assert_eq!(Flags.get_mode(), New_mode);

        let New_open = Open_type::New(true, true, false);
        let Flags = Flags.Set_open(New_open);
        assert_eq!(Flags.get_open(), New_open);

        let New_status = Status_type::New(false, true, false, true);
        let Flags = Flags.Set_status(New_status);
        assert_eq!(Flags.get_status(), New_status);
    }

    #[test]
    fn test_flags_type_is_permission_granted() {
        let Mode = Mode_type::READ_WRITE;
        let Status = Status_type::New(true, false, false, false);
        let Flags = Flags_type::New(Mode, None, Some(Status));

        assert!(Flags.is_permission_granted(&Permission_type::READ_ONLY));
        assert!(Flags.is_permission_granted(&Permission_type::WRITE_ONLY));
        assert!(Flags.is_permission_granted(&Permission_type::READ_WRITE));
    }

    #[test]
    fn test_flags_type_from_mode_type() {
        let Mode = Mode_type::READ_WRITE;
        let Flags: Flags_type = Mode.into();
        assert_eq!(Flags.get_mode(), Mode);
    }

    #[test]
    fn test_flags_type_into_u16() {
        let Mode = Mode_type::READ_WRITE;
        let Flags = Flags_type::New(Mode, None, None);
        let Flags_u16: u16 = Flags.into();
        assert_eq!(Flags_u16, Flags.0);
    }
}
