use std::sync::Mutex;
use std::time::Duration;

use super::lvgl;

use super::Point_type;
use File_system::File_type;

use crate::Display_type;
use crate::{Error_type, Input_type, Result_type, Screen_read_data_type};

/// Avoid using Arc, because the manager is a singleton.
static mut Manager_instance: Option<Manager_type> = None;

pub fn Initialize() -> Result_type<&'static Manager_type> {
    unsafe {
        if Is_initialized() {
            return Err(Error_type::Already_initialized);
        }

        Manager_instance.replace(Manager_type::New(Time::Get_instance())?);
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

impl Drop for Manager_type {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_deinit();
        }
    }
}

extern "C" fn Binding_tick_callback_function() -> u32 {
    Time::Get_instance().Get_current_time().as_millis() as u32
}

impl Manager_type {
    fn New(_: &Time::Manager_type) -> Result_type<Self> {
        unsafe {
            lvgl::lv_init();

            if !lvgl::lv_is_initialized() {
                panic!("Failed to initialize lvgl");
            }

            lvgl::lv_tick_set_cb(Some(Binding_tick_callback_function));
        }

        Ok(Self(Mutex::new(Inner(None))))
    }

    pub fn Loop() {
        loop {
            unsafe {
                let Time_untill_next = lvgl::lv_timer_handler();
                Task::Manager_type::Sleep(Duration::from_millis(Time_untill_next as u64));
            }
        }
    }

    pub fn Create_display<const Buffer_size: usize>(
        &self,
        Screen_file: File_type,
        Pointer_file: File_type,
        Double_buffered: bool,
    ) -> Result_type<Display_type<Buffer_size>> {
        let mut Screen_read_data = Screen_read_data_type::default();

        Screen_file
            .Read(Screen_read_data.as_mut())
            .map_err(|_| Error_type::Failed_to_get_resolution)?;

        let Resolution: Point_type = Screen_read_data.Get_resolution();

        let Display = Display_type::New(Screen_file, Resolution, Double_buffered)?;

        let Input = Input_type::New(Pointer_file, &Display)?;

        self.0.lock()?.0.replace(Input);

        Ok(Display)
    }
}
