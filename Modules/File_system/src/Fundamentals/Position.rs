/// Position type
///
/// This type is used to set the position in a file.
///
/// # Examples
///
/// ```rust
/// use File_system::Position_type;
///
/// let Position = Position_type::Start(0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position_type {
    Start(u64),
    Current(i64),
    End(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Whence_type {
    Start,
    Current,
    End,
}

impl Position_type {
    pub fn From_whence(Whence: Whence_type, Offset: i64) -> Self {
        match Whence {
            Whence_type::Start => Position_type::Start(Offset as u64),
            Whence_type::Current => Position_type::Current(Offset),
            Whence_type::End => Position_type::End(Offset),
        }
    }
}
