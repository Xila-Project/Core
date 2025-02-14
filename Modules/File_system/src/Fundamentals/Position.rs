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
