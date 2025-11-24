use core::fmt::Debug;

use shared::flags;

use super::Permission;

flags! {
    /// The flags for opening a file.
    pub enum AccessFlags: u8 {
        /// Read permission.
        Read,
        /// Write permission.
        Write,
    }
}

impl AccessFlags {
    pub const READ_WRITE: Self = Self::None.insert(Self::Read).insert(Self::Write);

    pub const fn into_permission(&self) -> Permission {
        let mut permission = Permission::None;

        if self.contains(AccessFlags::Read) {
            permission = permission.insert(Permission::Read);
        }

        if self.contains(AccessFlags::Write) {
            permission = permission.insert(Permission::Write);
        }

        permission
    }
}

flags! {
    /// The flags for opening a file.
    pub enum CreateFlags: u8 {
        /// Create the file if it does not exist.
        Create,
        /// Fail if the file already exists.
        Exclusive,
        /// Truncate the file to zero length if it exists.
        Truncate,
    }
}

impl CreateFlags {
    pub const CREATE_TRUNCATE: Self = Self::Create.insert(Self::Truncate);
    pub const CREATE_EXCLUSIVE: Self = Self::Create.insert(Self::Exclusive);
}

flags! {
    /// The status flags of a file.
    pub enum StateFlags: u8 {
        /// Append mode.
        Append,
        /// Non-blocking mode.
        NonBlocking,
        /// Synchronous mode.
        Synchronous,
        /// Synchronous data only mode.
        SynchronousDataOnly,
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
/// use file_system::{Flags, AccessFlags, CreateFlags, StateFlags};
///     
/// let flags = Flags::new(AccessFlags::READ_WRITE, Some(CreateFlags::Create), Some(StateFlags::NonBlocking));
///
/// assert_eq!(flags.get_access(), AccessFlags::READ_WRITE);
/// assert_eq!(flags.get_create(), CreateFlags::Create);
/// assert_eq!(flags.get_state(), StateFlags::NonBlocking);
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Flags(u16);

impl Debug for Flags {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Flags")
            .field("Access", &self.get_access())
            .field("Create", &self.get_create())
            .field("State", &self.get_state())
            .finish()
    }
}

impl Flags {
    const MODE_POSITION: u8 = 0;
    const OPEN_POSITION: u8 = AccessFlags::bits_used();
    const STATUS_POSITION: u8 = CreateFlags::bits_used() + Self::OPEN_POSITION;

    const OPEN_MASK: u16 = (1 << CreateFlags::bits_used()) - 1;
    const STATUS_MASK: u16 = (1 << StateFlags::bits_used()) - 1;
    const MODE_MASK: u16 = (1 << AccessFlags::bits_used()) - 1;

    pub const fn new(
        mode: AccessFlags,
        open: Option<CreateFlags>,
        status: Option<StateFlags>,
    ) -> Self {
        let open = if let Some(open_val) = open {
            open_val
        } else {
            CreateFlags::None
        };
        let status = if let Some(status_val) = status {
            status_val
        } else {
            StateFlags::None
        };

        let mut flags: u16 = 0;
        flags |= (mode.0 as u16) << Self::MODE_POSITION;
        flags |= (open.0 as u16) << Self::OPEN_POSITION;
        flags |= (status.0 as u16) << Self::STATUS_POSITION;
        Self(flags)
    }

    pub const fn get_access(&self) -> AccessFlags {
        AccessFlags(((self.0 >> Self::MODE_POSITION) & Self::MODE_MASK) as u8)
    }

    pub const fn get_create(&self) -> CreateFlags {
        CreateFlags(((self.0 >> Self::OPEN_POSITION) & Self::OPEN_MASK) as u8)
    }

    pub const fn get_state(&self) -> StateFlags {
        StateFlags(((self.0 >> Self::STATUS_POSITION) & Self::STATUS_MASK) as u8)
    }

    pub const fn set_mode(mut self, mode: AccessFlags) -> Self {
        self.0 &= !(Self::MODE_MASK << Self::MODE_POSITION);
        self.0 |= (mode.0 as u16) << Self::MODE_POSITION;
        self
    }

    pub const fn set_open(mut self, open: CreateFlags) -> Self {
        self.0 &= !(Self::OPEN_MASK << Self::OPEN_POSITION);
        self.0 |= (open.0 as u16) << Self::OPEN_POSITION;
        self
    }

    pub const fn set_status(mut self, status: StateFlags) -> Self {
        self.0 &= !(Self::STATUS_MASK << Self::STATUS_POSITION);
        self.0 |= (status.0 as u16) << Self::STATUS_POSITION;
        self
    }

    pub fn is_permission_granted(&self, permission: &Permission) -> bool {
        let mode = self.get_access();

        let read = permission.contains(Permission::Read) && mode.contains(AccessFlags::Read); // Read permission
        let write = permission.contains(Permission::Write)
            && (mode.contains(AccessFlags::Write) || self.get_state().contains(StateFlags::Append)); // Write permission

        read || write
    }

    pub fn split(&self) -> (AccessFlags, CreateFlags, StateFlags) {
        (self.get_access(), self.get_create(), self.get_state())
    }
}

impl From<AccessFlags> for Flags {
    fn from(mode: AccessFlags) -> Self {
        Self::new(mode, None, None)
    }
}

impl From<Flags> for u16 {
    fn from(flags: Flags) -> Self {
        flags.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_type_set_get() {
        let flags = Flags::new(AccessFlags::Read, None, None);

        let new_mode = AccessFlags::Write;
        let flags = flags.set_mode(new_mode);
        assert_eq!(flags.get_access(), new_mode);

        let new_open = CreateFlags::Exclusive | CreateFlags::Truncate;
        let flags = flags.set_open(new_open);
        assert_eq!(flags.get_create(), new_open);

        let new_status = StateFlags::NonBlocking | StateFlags::SynchronousDataOnly;
        let flags = flags.set_status(new_status);
        assert_eq!(flags.get_state(), new_status);
    }

    #[test]
    fn test_flags_type_is_permission_granted() {
        let mode = AccessFlags::READ_WRITE;
        let status = StateFlags::None;
        let flags = Flags::new(mode, None, Some(status));

        assert!(flags.is_permission_granted(&Permission::Read));
        assert!(flags.is_permission_granted(&Permission::Write));
        assert!(flags.is_permission_granted(&Permission::READ_WRITE));
    }

    #[test]
    fn test_flags_type_from_mode_type() {
        let mode = AccessFlags::READ_WRITE;
        let flags: Flags = mode.into();
        assert_eq!(flags.get_access(), mode);
    }

    #[test]
    fn test_flags_type_into_u16() {
        let mode = AccessFlags::READ_WRITE;
        let flags = Flags::new(mode, None, None);
        let flags_u16: u16 = flags.into();
        assert_eq!(flags_u16, flags.0);
    }
}
