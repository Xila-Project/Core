#[cfg(target_pointer_width = "32")]
pub type Entry_identifier_inner_type = u16;
#[cfg(target_pointer_width = "64")]
pub type Entry_identifier_inner_type = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Entry_identifier_type(Entry_identifier_inner_type);

impl Entry_identifier_type {
    pub const fn new(identifier: Entry_identifier_inner_type) -> Self {
        Self(identifier)
    }
}

impl From<Entry_identifier_inner_type> for Entry_identifier_type {
    fn from(internal_directory_entry_identifier: Entry_identifier_inner_type) -> Self {
        Entry_identifier_type(internal_directory_entry_identifier)
    }
}

impl From<Entry_identifier_type> for Entry_identifier_inner_type {
    fn from(internal_directory_entry_identifier: Entry_identifier_type) -> Self {
        internal_directory_entry_identifier.0
    }
}
