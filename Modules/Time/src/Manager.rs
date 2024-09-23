use std::time::Duration;

use crate::{Driver::Driver_trait, Error_type, Result_type};

pub static mut Manager: Option<Manager_type> = None;

pub fn Get_instance() -> &'static Manager_type {
    unsafe { Manager.as_ref().expect("Time manager is not initialized") }
}

pub fn Is_initialized() -> bool {
    unsafe { Manager.is_some() }
}

pub fn Initialize(Driver: Box<dyn Driver_trait>) -> Result_type<&'static Manager_type> {
    if Is_initialized() {
        return Err(Error_type::Already_initialized);
    }
    unsafe {
        Manager.replace(Manager_type::New(Driver)?);
    }

    Ok(Get_instance())
}

pub struct Manager_type {
    Driver: Box<dyn Driver_trait>,
}

impl Manager_type {
    pub fn New(Driver: Box<dyn Driver_trait>) -> Result_type<Self> {
        Ok(Self { Driver })
    }

    pub fn Get_current_time_since_startup(&self) -> Duration {
        self.Driver.Get_instant_since_startup()
    }

    pub fn Get_current_time(&self) -> Duration {
        self.Driver.Get_current_time()
    }
}
