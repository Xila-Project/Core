use File_system::Prelude::File_system_traits;

pub struct Data_type<'a> {
    File_system: &'a dyn File_system_traits,
}

impl Data_type<'_> {
    pub fn New(File_system: &dyn File_system_traits) -> Data_type<'_> {
        Data_type { File_system }
    }

    pub fn Get_file_system(&self) -> &dyn File_system_traits {
        self.File_system
    }
}
