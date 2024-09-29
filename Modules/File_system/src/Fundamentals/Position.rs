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
#[repr(C)]
pub enum Position_type {
    Start(u64),
    Current(i64),
    End(i64),
}

#[cfg(feature = "std")]
impl From<Position_type> for std::io::SeekFrom {
    fn from(Position: Position_type) -> Self {
        match Position {
            Position_type::Start(Item) => std::io::SeekFrom::Start(Item),
            Position_type::Current(Item) => std::io::SeekFrom::Current(Item),
            Position_type::End(Item) => std::io::SeekFrom::End(Item),
        }
    }
}
