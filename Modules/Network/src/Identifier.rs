#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Socket_identifier_type(usize);

impl Socket_identifier_type {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}
