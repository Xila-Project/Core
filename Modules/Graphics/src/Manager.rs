use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::time::Duration;

use File_system::Device_type;

use super::lvgl;

use super::Point_type;

use crate::Display_type;
use crate::{Error_type, Input_type, Result_type, Screen_read_data_type};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize() -> &'static Manager_type {
    Manager_instance.get_or_init(|| {
        Manager_type::New(Time::Get_instance()).expect("Failed to create manager instance")
    })
}

pub fn Get_instance() -> &'static Manager_type {
    Manager_instance
        .get()
        .expect("Failed to get manager instance")
}

struct Inner(Option<Input_type>);

pub struct Manager_type {
    Inner: Mutex<Inner>,
    Global_lock: Mutex<()>,
}

impl Drop for Manager_type {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_deinit();
        }
    }
}

extern "C" fn Binding_tick_callback_function() -> u32 {
    Time::Get_instance()
        .Get_current_time()
        .unwrap_or_default()
        .As_milliseconds() as u32
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

        Ok(Self {
            Inner: Mutex::new(Inner(None)),
            Global_lock: Mutex::new(()),
        })
    }

    pub fn Loop() {
        loop {
            unsafe {
                let Time_until_next = lvgl::lv_timer_handler();
                Task::Manager_type::Sleep(Duration::from_millis(Time_until_next as u64));
            }
        }
    }

    pub fn Create_display<const Buffer_size: usize>(
        &self,
        Screen_device: Device_type,
        Pointer_device: Device_type,
        Double_buffered: bool,
    ) -> Result_type<Display_type<Buffer_size>> {
        let mut Screen_read_data = Screen_read_data_type::default();

        Screen_device
            .Read(Screen_read_data.as_mut())
            .map_err(|_| Error_type::Failed_to_get_resolution)?;

        let Resolution: Point_type = Screen_read_data.Get_resolution();

        let Display = Display_type::New(Screen_device, Resolution, Double_buffered)?;

        let Input = Input_type::New(Pointer_device, &Display)?;

        self.Inner.lock()?.0.replace(Input);

        Ok(Display)
    }

    pub fn Lock(&self) -> Result_type<MutexGuard<'_, ()>> {
        Ok(self.Global_lock.lock()?)
    }
}
