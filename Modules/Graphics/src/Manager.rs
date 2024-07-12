use std::sync::Mutex;

use File_system::File_type;
use Screen::Prelude::{Point_type, Screen_traits};

use crate::{Display_type, Error_type, Input_type, Result_type};

/// Avoid using Arc, because the manager is a singleton.
static mut Manager_instance: Option<Manager_type> = None;

pub fn Initialize() -> Result_type<&'static Manager_type> {
    unsafe {
        if Is_initialized() {
            return Err(Error_type::Already_initialized);
        }

        Manager_instance.replace(Manager_type::New());
    }
    Get_instance()
}

pub fn Is_initialized() -> bool {
    unsafe { Manager_instance.is_some() }
}

pub fn Get_instance() -> Result_type<&'static Manager_type> {
    unsafe { Manager_instance.as_ref().ok_or(Error_type::Not_initialized) }
}

struct Inner(Option<Input_type>);

pub struct Manager_type(Mutex<Inner>);

impl Manager_type {
    fn New() -> Self {
        Self(Mutex::new(Inner(None)))
    }

    pub fn Create_display<const Buffer_size: usize>(
        &self,
        Screen: Box<dyn Screen_traits>,
        Resolution: Point_type,
        Input_path: File_type,
    ) -> Result_type<Display_type> {
        let Display = Display_type::New::<Buffer_size>(Screen, Resolution)?;

        let Input = Input_type::New(Input_path, &Display)?;

        self.0.lock()?.0.replace(Input);

        Ok(Display)
    }
}
