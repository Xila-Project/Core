use std::fmt::Debug;

use super::Permission_type;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Mode_type(u8);

impl Mode_type {
    pub const Read_bit: u8 = 1 << 0;
    pub const Write_bit: u8 = 1 << 1;

    pub const Size: u8 = 2;

    pub const Read_only: Self = Self::New(true, false);
    pub const Write_only: Self = Self::New(false, true);
    pub const Read_write: Self = Self::New(true, true);

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
        self.Set_bit(Self::Read_bit, Value)
    }

    pub const fn Set_write(self, Value: bool) -> Self {
        self.Set_bit(Self::Write_bit, Value)
    }

    pub const fn Get_bit(&self, Mask: u8) -> bool {
        self.0 & Mask != 0
    }

    pub const fn Get_read(&self) -> bool {
        self.Get_bit(Self::Read_bit)
    }

    pub const fn Get_write(&self) -> bool {
        self.Get_bit(Self::Write_bit)
    }
}

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

impl Status_type {
    pub const Append_bit: u8 = 1 << 0;
    pub const Non_blocking_bit: u8 = 1 << 1;
    pub const Synchronous_bit: u8 = 1 << 2;
    pub const Synchronous_data_only_bit: u8 = 1 << 3;

    pub const Size: u8 = 4;

    pub const None: Self = Self::New(false, false, false, false);

    const fn New(
        Append: bool,
        Non_blocking: bool,
        Synchronous: bool,
        Synchronous_data_only: bool,
    ) -> Self {
        Self(0)
            .Set_append(Append)
            .Set_non_blocking(Non_blocking)
            .Set_synchronous(Synchronous)
            .Set_synchronous_data_only(Synchronous_data_only)
    }

    const fn Set_bit(mut self, Mask: u8, Value: bool) -> Self {
        if Value {
            self.0 |= Mask;
        } else {
            self.0 &= !Mask;
        }
        self
    }

    const fn Get_bit(&self, Mask: u8) -> bool {
        self.0 & Mask != 0
    }

    pub const fn Set_non_blocking(self, Value: bool) -> Self {
        self.Set_bit(Self::Non_blocking_bit, Value)
    }

    pub fn Get_non_blocking(&self) -> bool {
        self.Get_bit(Self::Non_blocking_bit)
    }

    pub const fn Set_append(self, Value: bool) -> Self {
        self.Set_bit(Self::Append_bit, Value)
    }

    pub const fn Get_append(&self) -> bool {
        self.Get_bit(Self::Append_bit)
    }

    pub const fn Set_synchronous(self, Value: bool) -> Self {
        self.Set_bit(Self::Synchronous_bit, Value)
    }

    pub const fn Get_synchronous(&self) -> bool {
        self.Get_bit(Self::Synchronous_bit)
    }

    pub const fn Set_synchronous_data_only(self, Value: bool) -> Self {
        self.Set_bit(Self::Synchronous_data_only_bit, Value)
    }

    pub const fn Get_synchronous_data_only(&self) -> bool {
        self.Get_bit(Self::Synchronous_data_only_bit)
    }
}

impl Debug for Status_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Status_type")
            .field("Append", &self.Get_append())
            .field("Non_blocking", &self.Get_non_blocking())
            .field("Synchronous", &self.Get_bit(Self::Synchronous_bit))
            .field(
                "Synchronous_data_only",
                &self.Get_bit(Self::Synchronous_data_only_bit),
            )
            .finish()
    }
}

impl Default for Status_type {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Flags_type(u16);

impl Flags_type {
    const Mode_position: u8 = 0;
    const Open_position: u8 = Mode_type::Size;
    const Status_position: u8 = Open_type::Size + Self::Open_position;

    const Open_mask: u16 = (1 << Open_type::Size) - 1;
    const Status_mask: u16 = (1 << Status_type::Size) - 1;
    const Mode_mask: u16 = (1 << Mode_type::Size) - 1;

    pub const fn New(
        Mode: Mode_type,
        Open: Option<Open_type>,
        Status: Option<Status_type>,
    ) -> Self {
        let Open = if let Some(Open) = Open {
            Open
        } else {
            Open_type::None
        };
        let Status = if let Some(Status) = Status {
            Status
        } else {
            Status_type::None
        };

        let mut Flags: u16 = 0;
        Flags |= (Mode.0 as u16) << Self::Mode_position;
        Flags |= (Open.0 as u16) << Self::Open_position;
        Flags |= (Status.0 as u16) << Self::Status_position;
        Self(Flags)
    }

    pub const fn Get_mode(&self) -> Mode_type {
        Mode_type(((self.0 >> Self::Mode_position) & Self::Mode_mask) as u8)
    }

    pub const fn Get_open(&self) -> Open_type {
        Open_type(((self.0 >> Self::Open_position) & Self::Open_mask) as u8)
    }

    pub const fn Get_status(&self) -> Status_type {
        Status_type(((self.0 >> Self::Status_position) & Self::Status_mask) as u8)
    }

    pub const fn Set_mode(mut self, Mode: Mode_type) -> Self {
        self.0 &= !(Self::Mode_mask << Self::Mode_position);
        self.0 |= (Mode.0 as u16) << Self::Mode_position;
        self
    }

    pub const fn Set_open(mut self, Open: Open_type) -> Self {
        self.0 &= !(Self::Open_mask << Self::Open_position);
        self.0 |= (Open.0 as u16) << Self::Open_position;
        self
    }

    pub const fn Set_status(mut self, Status: Status_type) -> Self {
        self.0 &= !(Self::Status_mask << Self::Status_position);
        self.0 |= (Status.0 as u16) << Self::Status_position;
        self
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
        Self::New(Mode, None, None)
    }
}

impl From<Flags_type> for u16 {
    fn from(Flags: Flags_type) -> Self {
        Flags.0
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_mode_type_new() {
        let Read_only = Mode_type::New(true, false);
        assert_eq!(Read_only.Get_read(), true);
        assert_eq!(Read_only.Get_write(), false);

        let Write_only = Mode_type::New(false, true);
        assert_eq!(Write_only.Get_read(), false);
        assert_eq!(Write_only.Get_write(), true);

        let Read_write = Mode_type::New(true, true);
        assert_eq!(Read_write.Get_read(), true);
        assert_eq!(Read_write.Get_write(), true);
    }

    #[test]
    fn Test_mode_type_set_get() {
        let mut Mode = Mode_type(0);
        Mode = Mode.Set_read(true);
        assert_eq!(Mode.Get_read(), true);
        assert_eq!(Mode.Get_write(), false);

        Mode = Mode.Set_write(true);
        assert_eq!(Mode.Get_read(), true);
        assert_eq!(Mode.Get_write(), true);

        Mode = Mode.Set_read(false);
        assert_eq!(Mode.Get_read(), false);
        assert_eq!(Mode.Get_write(), true);
    }

    #[test]
    fn Test_open_type_new() {
        let Open = Open_type::New(true, false, true, false);
        assert_eq!(Open.Get_create(), true);
        assert_eq!(Open.Get_create_only(), false);
        assert_eq!(Open.Get_truncate(), true);
        assert_eq!(Open.Get_directory(), false);
    }

    #[test]
    fn Test_open_type_set_get() {
        let mut Open = Open_type(0);
        Open = Open.Set_create(true);
        assert_eq!(Open.Get_create(), true);
        assert_eq!(Open.Get_create_only(), false);

        Open = Open.Set_create_only(true);
        assert_eq!(Open.Get_create(), true);
        assert_eq!(Open.Get_create_only(), true);

        Open = Open.Set_truncate(true);
        assert_eq!(Open.Get_truncate(), true);

        Open = Open.Set_directory(true);
        assert_eq!(Open.Get_directory(), true);
    }

    #[test]
    fn Test_status_type_new() {
        let Status = Status_type::New(true, false, true, false);
        assert_eq!(Status.Get_append(), true);
        assert_eq!(Status.Get_non_blocking(), false);
        assert_eq!(Status.Get_synchronous(), true);
        assert_eq!(Status.Get_synchronous_data_only(), false);
    }

    #[test]
    fn Test_status_type_set_get() {
        let mut Status = Status_type(0);
        Status = Status.Set_append(true);
        assert_eq!(Status.Get_append(), true);
        assert_eq!(Status.Get_non_blocking(), false);

        Status = Status.Set_non_blocking(true);
        assert_eq!(Status.Get_non_blocking(), true);

        Status = Status.Set_synchronous(true);
        assert_eq!(Status.Get_synchronous(), true);

        Status = Status.Set_synchronous_data_only(true);
        assert_eq!(Status.Get_synchronous_data_only(), true);
    }

    #[test]
    fn Test_flags_type_new() {
        let Mode = Mode_type::Read_write;
        let Open = Open_type::New(true, false, true, false);
        let Status = Status_type::New(true, false, true, false);

        let Flags = Flags_type::New(Mode, Some(Open), Some(Status));
        assert_eq!(Flags.Get_mode(), Mode);
        assert_eq!(Flags.Get_open(), Open);
        assert_eq!(Flags.Get_status(), Status);
    }

    #[test]
    fn Test_flags_type_set_get() {
        let Flags = Flags_type::New(Mode_type::Read_only, None, None);

        let New_mode = Mode_type::Write_only;
        let Flags = Flags.Set_mode(New_mode);
        assert_eq!(Flags.Get_mode(), New_mode);

        let New_open = Open_type::New(true, true, false, false);
        let Flags = Flags.Set_open(New_open);
        assert_eq!(Flags.Get_open(), New_open);

        let New_status = Status_type::New(false, true, false, true);
        let Flags = Flags.Set_status(New_status);
        assert_eq!(Flags.Get_status(), New_status);
    }

    #[test]
    fn Test_flags_type_is_permission_granted() {
        let Mode = Mode_type::Read_write;
        let Status = Status_type::New(true, false, false, false);
        let Flags = Flags_type::New(Mode, None, Some(Status));

        assert_eq!(
            Flags.Is_permission_granted(&Permission_type::Read_only),
            true
        );
        assert_eq!(
            Flags.Is_permission_granted(&Permission_type::Write_only),
            true
        );
        assert_eq!(
            Flags.Is_permission_granted(&Permission_type::Read_write),
            true
        );
    }

    #[test]
    fn Test_flags_type_from_mode_type() {
        let Mode = Mode_type::Read_write;
        let Flags: Flags_type = Mode.into();
        assert_eq!(Flags.Get_mode(), Mode);
    }

    #[test]
    fn Test_flags_type_into_u16() {
        let Mode = Mode_type::Read_write;
        let Flags = Flags_type::New(Mode, None, None);
        let Flags_u16: u16 = Flags.into();
        assert_eq!(Flags_u16, Flags.0);
    }
}
