#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Type_type {
    File,
    Directory,
    Block_device,
    Character_device,
    Pipe,
    Socket,
    Symbolic_link,
}
