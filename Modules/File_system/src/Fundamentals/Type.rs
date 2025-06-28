use core::fmt::Display;

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

impl Display for Type_type {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let Type = match self {
            Type_type::File => "File",
            Type_type::Directory => "Directory",
            Type_type::Block_device => "Block device",
            Type_type::Character_device => "Character device",
            Type_type::Pipe => "Pipe",
            Type_type::Socket => "Socket",
            Type_type::Symbolic_link => "Symbolic link",
        };

        write!(f, "{Type}")
    }
}
