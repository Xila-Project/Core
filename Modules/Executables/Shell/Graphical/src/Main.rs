use core::num::NonZeroUsize;

use Executable::Standard_type;

use crate::Shell_type;

pub fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Shell_type::New(Standard).Main(Arguments)
}

impl Shell_type {
    pub fn New(Standard: Standard_type) -> Self {
        let User: String = "".to_string();

        Self {
            Standard,
            User,
            Running: true,
        }
    }

    pub fn Main(&mut self, Arguments: String) -> Result<(), NonZeroUsize> {
        self.Standard.Print_line("Hello, World!");

        let Window = Graphics::Get_instance().Create_window();

        Ok(())
    }
}
