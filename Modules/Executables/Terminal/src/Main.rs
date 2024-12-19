use core::num::NonZeroUsize;

use Executable::Standard_type;
use Graphics::Window_type;

use crate::Error::Result_type;

pub struct Terminal_type {
    _Standard: Standard_type,
    Window: Window_type,
}

impl Terminal_type {
    pub fn New(Standard: Standard_type) -> Result_type<Self> {
        let Window = Graphics::Get_instance().Create_window()?;

        Ok(Self {
            _Standard: Standard,
            Window,
        })
    }

    pub fn Main(&mut self, _: String) -> Result<(), NonZeroUsize> {
        Ok(())
    }
}

pub fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Terminal_type::New(Standard)?.Main(Arguments)
}
