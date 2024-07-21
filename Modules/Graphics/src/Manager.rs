use std::sync::Mutex;

use super::Point_type;
use File_system::File_type;

use crate::{Display_type, Error_type, Input_type, Result_type, Screen_read_data_type};

/// Avoid using Arc, because the manager is a singleton.
static mut Manager_instance: Option<Manager_type> = None;

pub fn Initialize() -> Result_type<&'static Manager_type> {
    unsafe {
        if Is_initialized() {
            return Err(Error_type::Already_initialized);
        }

        lvgl::init();

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
        Screen_file: File_type,
        Pointer_file: File_type,
    ) -> Result_type<Display_type> {
        let mut Screen_read_data = Screen_read_data_type::default();

        Screen_file
            .Read(Screen_read_data.as_mut())
            .map_err(|_| Error_type::Failed_to_get_resolution)?;

        let Resolution: Point_type = Screen_read_data.Get_resolution();

        let Display = Display_type::New::<Buffer_size>(Screen_file, Resolution)?;

        let Input = Input_type::New(Pointer_file, &Display)?;

        self.0.lock()?.0.replace(Input);

        Ok(Display)
    }
}
