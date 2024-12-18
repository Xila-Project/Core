use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::sync::RwLock;
use std::time::Duration;
use File_system::Device_type;

use super::LVGL;

use super::Point_type;

use crate::Display_type;
use crate::Input_type_type;
use crate::Window::Window_type;
use crate::{Error_type, Input_type, Result_type, Screen_read_data_type};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Initialize(
    Screen_device: Device_type,
    Input_device: Device_type,
    Input_device_type: Input_type_type,
    Buffer_size: usize,
    Double_buffered: bool,
) {
    Manager_instance
        .set(
            Manager_type::New(
                Time::Get_instance(),
                Screen_device,
                Input_device,
                Input_device_type,
                Buffer_size,
                Double_buffered,
            )
            .expect("Failed to create manager instance"),
        )
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

struct Inner_type {
    _Inputs: Vec<Input_type>,
    _Displays: Vec<Display_type>,
    Window_parent: *mut LVGL::lv_obj_t,
}

pub struct Manager_type {
    Inner: RwLock<Inner_type>,
    Global_lock: Mutex<()>,
}

impl Drop for Manager_type {
    fn drop(&mut self) {
        unsafe {
            LVGL::lv_deinit();
        }
    }
}

extern "C" fn Binding_tick_callback_function() -> u32 {
    Time::Get_instance()
        .Get_current_time()
        .unwrap_or_default()
        .As_milliseconds() as u32
}

unsafe impl Send for Manager_type {}

unsafe impl Sync for Manager_type {}

impl Manager_type {
    fn New(
        _: &Time::Manager_type,
        Screen_device: Device_type,
        Input_device: Device_type,
        Input_device_type: Input_type_type,
        Buffer_size: usize,
        Double_buffered: bool,
    ) -> Result_type<Self> {
        unsafe {
            LVGL::lv_init();

            if !LVGL::lv_is_initialized() {
                panic!("Failed to initialize lvgl");
            }

            LVGL::lv_tick_set_cb(Some(Binding_tick_callback_function));
        }

        let (Display, Input) = Self::Create_display(
            Screen_device,
            Buffer_size,
            Input_device,
            Input_device_type,
            Double_buffered,
        )?;

        let Screen = Display.Get_object();

        Ok(Self {
            Inner: RwLock::new(Inner_type {
                _Inputs: vec![Input],
                _Displays: vec![Display],
                Window_parent: Screen,
            }),
            Global_lock: Mutex::new(()),
        })
    }

    fn Loop(&self) -> Result_type<()> {
        loop {
            let Time_until_next = unsafe {
                let _Lock = self.Global_lock.lock()?;
                LVGL::lv_timer_handler()
            };
            Task::Manager_type::Sleep(Duration::from_millis(Time_until_next as u64));
        }
    }

    pub fn Set_window_parent(&self, Window_parent: *mut LVGL::lv_obj_t) -> Result_type<()> {
        self.Inner.write()?.Window_parent = Window_parent;

        Ok(())
    }

    pub fn Create_window(&self) -> Result_type<Window_type> {
        let Parent_object = self.Inner.write()?.Window_parent;

        let Window = unsafe { Window_type::New(Parent_object)? };

        Ok(Window)
    }

    pub fn Add_input_device(
        &self,
        Input_device: Device_type,
        Input_type: Input_type_type,
    ) -> Result_type<()> {
        let Input = Input_type::New(Input_device, Input_type)?;

        self.Inner.write()?._Inputs.push(Input);

        Ok(())
    }

    fn Create_display(
        Screen_device: Device_type,
        Buffer_size: usize,
        Input_device: Device_type,
        Input_device_type: Input_type_type,
        Double_buffered: bool,
    ) -> Result_type<(Display_type, Input_type)> {
        let mut Screen_read_data = Screen_read_data_type::default();

        Screen_device
            .Read(Screen_read_data.as_mut())
            .map_err(|_| Error_type::Failed_to_get_resolution)?;

        let Resolution: Point_type = Screen_read_data.Get_resolution();

        let Display = Display_type::New(Screen_device, Resolution, Buffer_size, Double_buffered)?;

        let Input = Input_type::New(Input_device, Input_device_type)?;

        Ok((Display, Input))
    }

    pub fn Lock(&self) -> Result_type<MutexGuard<'_, ()>> {
        Ok(self.Global_lock.lock()?)
    }

    pub fn Get_current_screen(&self) -> Result_type<*mut LVGL::lv_obj_t> {
        let _Lock = self.Lock()?;

        Ok(unsafe { LVGL::lv_screen_active() })
    }
}
