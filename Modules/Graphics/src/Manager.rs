use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::time::Duration;
use File_system::Device_type;

use super::lvgl;

use super::Point_type;

use crate::Display_type;
use crate::Window::Window_type;
use crate::{Error_type, Input_type, Result_type, Screen_read_data_type};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize() {
    Manager_instance
        .set(Manager_type::New(Time::Get_instance()).expect("Failed to create manager instance"))
        .map_err(|_| ())
        .expect(
            "
         Graphics manager was already initialized
        ",
        );

    let Task_instance = Task::Get_instance();

    Task_instance
        .New_thread(
            Task_instance.Get_current_task_identifier().unwrap(),
            "Graphics",
            None,
            move || {
                Get_instance().Loop().unwrap();
            },
        )
        .unwrap();
}

pub fn Get_instance() -> &'static Manager_type {
    Manager_instance
        .get()
        .expect("Failed to get manager instance")
}

struct Inner_type(Option<Input_type>);

pub struct Manager_type {
    Inner: Mutex<Inner_type>,
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
            Inner: Mutex::new(Inner_type(None)),
            Global_lock: Mutex::new(()),
        })
    }

    fn Loop(&self) -> Result_type<()> {
        loop {
            let Time_until_next = unsafe {
                let _Lock = self.Global_lock.lock()?;
                lvgl::lv_timer_handler()
            };
            Task::Manager_type::Sleep(Duration::from_millis(Time_until_next as u64));
        }
    }

    pub fn Create_window(&self) -> Result_type<Window_type> {
        let Parent_object = unsafe { lvgl::lv_screen_active() };

        let Window = unsafe { Window_type::New(Parent_object)? };

        Ok(Window)
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
