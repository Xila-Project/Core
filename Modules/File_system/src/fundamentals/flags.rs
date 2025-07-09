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
/// let mode = Mode_type::New(true, false);
///
/// assert_eq!(mode.get_read(), true);
/// assert_eq!(mode.get_write(), false);
///
/// let mode = Mode_type::New(false, true);
///
/// assert_eq!(mode.get_read(), false);
/// assert_eq!(mode.get_write(), true);
///
/// let mode = Mode_type::New(true, true);
///
/// assert_eq!(mode.get_read(), true);
/// assert_eq!(mode.get_write(), true);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Mode_type(u8);

impl Mode_type {
    pub const READ_BIT: u8 = 1 << 0;
    pub const WRITE_BIT: u8 = 1 << 1;

    pub const SIZE: u8 = 2;

    pub const READ_ONLY: Self = Self::new(true, false);
    pub const WRITE_ONLY: Self = Self::new(false, true);
    pub const READ_WRITE: Self = Self::new(true, true);

    pub const fn new(read: bool, write: bool) -> Self {
        Self(0).set_read(read).set_write(write)
    }

    pub const fn set_bit(mut self, mask: u8, value: bool) -> Self {
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
        self
    }

    pub const fn set_read(self, value: bool) -> Self {
        self.set_bit(Self::READ_BIT, value)
    }

    pub const fn set_write(self, value: bool) -> Self {
        self.set_bit(Self::WRITE_BIT, value)
    }

    pub const fn get_bit(&self, mask: u8) -> bool {
        self.0 & mask != 0
    }

    pub const fn get_read(&self) -> bool {
        self.get_bit(Self::READ_BIT)
    }

    pub const fn get_write(&self) -> bool {
        self.get_bit(Self::WRITE_BIT)
    }

    pub const fn from_u8(value: u8) -> Self {
        Self(value)
    }

    pub const fn as_u8(&self) -> u8 {
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
/// let open = Open_type::New(true, true, false);
///
/// assert_eq!(open.get_create(), true);
/// assert_eq!(open.get_exclusive(), true);
/// assert_eq!(open.get_truncate(), false);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Open_type(u8);

impl Open_type {
    pub const CREATE_MASK: u8 = 1 << 0;
    pub const EXCLUSIVE_MASK: u8 = 1 << 1;
    pub const TRUNCATE_MASK: u8 = 1 << 2;

    pub const SIZE: u8 = 3;

    pub const NONE: Self = Self::new(false, false, false);

    pub const CREATE: Self = Self::new(true, false, false);
    pub const CREATE_ONLY: Self = Self::new(true, true, false);
    pub const TRUNCATE: Self = Self::new(false, false, true);

    pub const fn new(create: bool, create_only: bool, truncate: bool) -> Self {
        Self(0)
            .set_create(create)
            .set_exclusive(create_only)
            .set_truncate(truncate)
    }

    pub const fn get_bit(&self, mask: u8) -> bool {
        self.0 & mask != 0
    }

    pub const fn set_bit(mut self, mask: u8, value: bool) -> Self {
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
        self
    }

    pub const fn get_create(&self) -> bool {
        self.get_bit(Self::CREATE_MASK)
    }

    pub const fn set_create(self, value: bool) -> Self {
        self.set_bit(Self::CREATE_MASK, value)
    }

    pub const fn get_exclusive(&self) -> bool {
        self.get_bit(Self::EXCLUSIVE_MASK)
    }

    pub const fn set_exclusive(self, value: bool) -> Self {
        self.set_bit(Self::EXCLUSIVE_MASK, value)
    }

    pub const fn get_truncate(&self) -> bool {
        self.get_bit(Self::TRUNCATE_MASK)
    }

    pub const fn set_truncate(self, value: bool) -> Self {
        self.set_bit(Self::TRUNCATE_MASK, value)
    }

    pub const fn from_u8(value: u8) -> Self {
        Self(value)
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
/// let status = Status_type::New(true, false, true, false);
///
/// assert_eq!(status.get_append(), true);
/// assert_eq!(status.get_non_blocking(), false);
/// assert_eq!(status.get_synchronous(), true);
/// assert_eq!(status.get_synchronous_data_only(), false);
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

    pub const NON_BLOCKING: Self = Self::new(false, true, false, false);

    pub const NONE: Self = Self::new(false, false, false, false);

    pub const fn new(
        append: bool,
        non_blocking: bool,
        synchronous: bool,
        synchronous_data_only: bool,
    ) -> Self {
        Self(0)
            .set_append(append)
            .set_non_blocking(non_blocking)
            .set_synchronous(synchronous)
            .set_synchronous_data_only(synchronous_data_only)
    }

    const fn set_bit(mut self, mask: u8, value: bool) -> Self {
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
        self
    }

    const fn get_bit(&self, mask: u8) -> bool {
        self.0 & mask != 0
    }

    pub const fn set_non_blocking(self, value: bool) -> Self {
        self.set_bit(Self::NON_BLOCKING_BIT, value)
    }

    pub fn get_non_blocking(&self) -> bool {
        self.get_bit(Self::NON_BLOCKING_BIT)
    }

    pub const fn set_append(self, value: bool) -> Self {
        self.set_bit(Self::APPEND_BIT, value)
    }

    pub const fn get_append(&self) -> bool {
        self.get_bit(Self::APPEND_BIT)
    }

    pub const fn set_synchronous(self, value: bool) -> Self {
        self.set_bit(Self::SYNCHRONOUS_BIT, value)
    }

    pub const fn get_synchronous(&self) -> bool {
        self.get_bit(Self::SYNCHRONOUS_BIT)
    }

    pub const fn set_synchronous_data_only(self, value: bool) -> Self {
        self.set_bit(Self::SYNCHRONOUS_DATA_ONLY_BIT, value)
    }

    pub const fn get_synchronous_data_only(&self) -> bool {
        self.get_bit(Self::SYNCHRONOUS_DATA_ONLY_BIT)
    }

    pub const fn from_u8(value: u8) -> Self {
        Self(value)
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
/// let flags = Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), Some(Status_type::Non_blocking));
///
/// assert_eq!(flags.get_mode(), Mode_type::Read_write);
/// assert_eq!(flags.get_open(), Open_type::Create_only);
/// assert_eq!(flags.get_status(), Status_type::Non_blocking);
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

    pub const fn new(
        mode: Mode_type,
        open: Option<Open_type>,
        status: Option<Status_type>,
    ) -> Self {
        let open = if let Some(open_val) = open {
            open_val
        } else {
            Open_type::NONE
        };
        let status = if let Some(status_val) = status {
            status_val
        } else {
            Status_type::NONE
        };

        let mut flags: u16 = 0;
        flags |= (mode.0 as u16) << Self::MODE_POSITION;
        flags |= (open.0 as u16) << Self::OPEN_POSITION;
        flags |= (status.0 as u16) << Self::STATUS_POSITION;
        Self(flags)
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

    pub const fn set_mode(mut self, mode: Mode_type) -> Self {
        self.0 &= !(Self::MODE_MASK << Self::MODE_POSITION);
        self.0 |= (mode.0 as u16) << Self::MODE_POSITION;
        self
    }

    pub const fn set_open(mut self, open: Open_type) -> Self {
        self.0 &= !(Self::OPEN_MASK << Self::OPEN_POSITION);
        self.0 |= (open.0 as u16) << Self::OPEN_POSITION;
        self
    }

    pub const fn set_status(mut self, status: Status_type) -> Self {
        self.0 &= !(Self::STATUS_MASK << Self::STATUS_POSITION);
        self.0 |= (status.0 as u16) << Self::STATUS_POSITION;
        self
    }

    pub fn is_permission_granted(&self, permission: &Permission_type) -> bool {
        let mode = self.get_mode();

        (permission.get_read() && mode.get_read()) // Read permission
            || (permission.get_write() && (mode.get_write() || self.get_status().get_append()))
        // Write permission
    }
}

impl From<Mode_type> for Flags_type {
    fn from(mode: Mode_type) -> Self {
        Self::new(mode, None, None)
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
        let read_only = Mode_type::new(true, false);
        assert!(read_only.get_read());
        assert!(!read_only.get_write());

        let write_only = Mode_type::new(false, true);
        assert!(!write_only.get_read());
        assert!(write_only.get_write());

        let read_write = Mode_type::new(true, true);
        assert!(read_write.get_read());
        assert!(read_write.get_write());
    }

    #[test]
    fn test_mode_type_set_get() {
        let mut mode = Mode_type(0);
        mode = mode.set_read(true);
        assert!(mode.get_read());
        assert!(!mode.get_write());

        mode = mode.set_write(true);
        assert!(mode.get_read());
        assert!(mode.get_write());

        mode = mode.set_read(false);
        assert!(!mode.get_read());
        assert!(mode.get_write());
    }

    #[test]
    fn test_open_type_new() {
        let open = Open_type::new(true, false, true);
        assert!(open.get_create());
        assert!(!open.get_exclusive());
        assert!(open.get_truncate());
    }

    #[test]
    fn test_open_type_set_get() {
        let mut open = Open_type(0);
        open = open.set_create(true);
        assert!(open.get_create());
        assert!(!open.get_exclusive());

        open = open.set_exclusive(true);
        assert!(open.get_create());
        assert!(open.get_exclusive());

        open = open.set_truncate(true);
        assert!(open.get_truncate());
    }

    #[test]
    fn test_status_type_new() {
        let status = Status_type::new(true, false, true, false);
        assert!(status.get_append());
        assert!(!status.get_non_blocking());
        assert!(status.get_synchronous());
        assert!(!status.get_synchronous_data_only());
    }

    #[test]
    fn test_status_type_set_get() {
        let mut status = Status_type(0);
        status = status.set_append(true);
        assert!(status.get_append());
        assert!(!status.get_non_blocking());

        status = status.set_non_blocking(true);
        assert!(status.get_non_blocking());

        status = status.set_synchronous(true);
        assert!(status.get_synchronous());

        status = status.set_synchronous_data_only(true);
        assert!(status.get_synchronous_data_only());
    }

    #[test]
    fn test_flags_type_new() {
        let mode = Mode_type::READ_WRITE;
        let open = Open_type::new(true, false, true);
        let status = Status_type::new(true, false, true, false);

        let flags = Flags_type::new(mode, Some(open), Some(status));
        assert_eq!(flags.get_mode(), mode);
        assert_eq!(flags.get_open(), open);
        assert_eq!(flags.get_status(), status);
    }

    #[test]
    fn test_flags_type_set_get() {
        let flags = Flags_type::new(Mode_type::READ_ONLY, None, None);

        let new_mode = Mode_type::WRITE_ONLY;
        let flags = flags.set_mode(new_mode);
        assert_eq!(flags.get_mode(), new_mode);

        let new_open = Open_type::new(true, true, false);
        let flags = flags.set_open(new_open);
        assert_eq!(flags.get_open(), new_open);

        let new_status = Status_type::new(false, true, false, true);
        let flags = flags.set_status(new_status);
        assert_eq!(flags.get_status(), new_status);
    }

    #[test]
    fn test_flags_type_is_permission_granted() {
        let mode = Mode_type::READ_WRITE;
        let status = Status_type::new(true, false, false, false);
        let flags = Flags_type::new(mode, None, Some(status));

        assert!(flags.is_permission_granted(&Permission_type::READ_ONLY));
        assert!(flags.is_permission_granted(&Permission_type::WRITE_ONLY));
        assert!(flags.is_permission_granted(&Permission_type::READ_WRITE));
    }

    #[test]
    fn test_flags_type_from_mode_type() {
        let mode = Mode_type::READ_WRITE;
        let flags: Flags_type = mode.into();
        assert_eq!(flags.get_mode(), mode);
    }

    #[test]
    fn test_flags_type_into_u16() {
        let mode = Mode_type::READ_WRITE;
        let flags = Flags_type::new(mode, None, None);
        let flags_u16: u16 = flags.into();
        assert_eq!(flags_u16, flags.0);
    }
}
