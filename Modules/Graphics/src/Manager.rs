extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use Synchronization::blocking_mutex::raw::CriticalSectionRawMutex;
use Synchronization::mutex::{Mutex, MutexGuard};
use Synchronization::{once_lock::OnceLock, rwlock::RwLock};

use core::mem::forget;

use core::time::Duration;
use File_system::Device_type;

use super::LVGL;

use super::Point_type;

use crate::Color_type;
use crate::Display_type;
use crate::Input_type;
use crate::Input_type_type;
use crate::Window::Window_type;
use crate::{Error_type, Result_type, Screen_read_data_type};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub async fn Initialize(
    Screen_device: Device_type,
    Input_device: Device_type,
    Input_device_type: Input_type_type,
    Buffer_size: usize,
    Double_buffered: bool,
) -> &'static Manager_type {
    let Manager = Manager_type::New(
        Time::Get_instance(),
        Screen_device,
        Input_device,
        Input_device_type,
        Buffer_size,
        Double_buffered,
    )
    .expect("Failed to create manager instance");

    let Instance = Manager_instance.get_or_init(|| Manager);

    let Task_instance = Task::Get_instance();

    Task_instance
        .Spawn(
            Task_instance.Get_current_task_identifier().await,
            "Graphics",
            None,
            async move |_| {
                let _ = Get_instance().await.Loop().await;
            },
        )
        .await
        .expect("Failed to spawn graphics task");

    Instance
}

pub async fn Get_instance() -> &'static Manager_type {
    Manager_instance.get().await
}

struct Inner_type {
    _Inputs: Vec<Input_type>,
    _Displays: Vec<Display_type>,
    Window_parent: *mut LVGL::lv_obj_t,
}

pub struct Manager_type {
    Inner: RwLock<CriticalSectionRawMutex, Inner_type>,
    Global_lock: Mutex<CriticalSectionRawMutex, ()>,
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

        unsafe {
            let Group = LVGL::lv_group_create();
            LVGL::lv_group_set_default(Group);
        }

        Ok(Self {
            Inner: RwLock::new(Inner_type {
                _Inputs: vec![Input],
                _Displays: vec![Display],

                Window_parent: Screen,
            }),
            Global_lock: Mutex::new(()),
        })
    }

    async fn Loop(&self) -> Result_type<()> {
        loop {
            let Time_until_next = unsafe {
                let _Lock = self.Global_lock.lock().await;
                LVGL::lv_timer_handler()
            };
            Task::Manager_type::Sleep(Duration::from_millis(Time_until_next as u64)).await;
        }
    }

    pub async fn Set_window_parent(&self, Window_parent: *mut LVGL::lv_obj_t) -> Result_type<()> {
        self.Inner.write().await.Window_parent = Window_parent;

        Ok(())
    }

    pub async fn Create_window(&self) -> Result_type<Window_type> {
        let Parent_object = self.Inner.write().await.Window_parent;

        let Window = unsafe { Window_type::New(Parent_object)? };

        Ok(Window)
    }

    pub async fn Add_input_device(
        &self,
        Input_device: Device_type,
        Input_type: Input_type_type,
    ) -> Result_type<()> {
        let Input = Input_type::New(Input_device, Input_type)?;

        self.Inner.write().await._Inputs.push(Input);

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

    pub async fn Get_window_count(&self) -> Result_type<usize> {
        let Window_parent = self.Inner.read().await.Window_parent;
        unsafe { Ok(LVGL::lv_obj_get_child_count(Window_parent) as usize) }
    }

    pub async fn Get_window_icon(&self, Index: usize) -> Result_type<(String, Color_type)> {
        let Window_parent = self.Inner.read().await.Window_parent;

        let Window = unsafe {
            let Child = LVGL::lv_obj_get_child(Window_parent, Index as i32);

            Window_type::From_raw(Child)
        };

        let Icon = Window.Get_icon();

        let Icon = (Icon.0.to_string(), Icon.1);

        forget(Window);

        Ok(Icon)
    }

    pub async fn Get_window_identifier(&self, Index: usize) -> Result_type<usize> {
        let Window_parent = self.Inner.read().await.Window_parent;

        let Window = unsafe { LVGL::lv_obj_get_child(Window_parent, Index as i32) as usize };

        Ok(Window)
    }

    pub async fn Maximize_window(&self, Identifier: usize) -> Result_type<()> {
        let Window_count = self.Get_window_count().await?;

        let Window_parent = self.Inner.read().await.Window_parent;

        let Found = (0..Window_count).find(|Index| unsafe {
            let Child = LVGL::lv_obj_get_child(Window_parent, *Index as i32);

            Child == Identifier as *mut LVGL::lv_obj_t
        });

        if Found.is_some() {
            unsafe {
                LVGL::lv_obj_move_foreground(Identifier as *mut LVGL::lv_obj_t);
            }

            Ok(())
        } else {
            Err(Error_type::Invalid_window_identifier)
        }
    }

    pub async fn Lock(&self) -> Result_type<MutexGuard<'_, CriticalSectionRawMutex, ()>> {
        Ok(self.Global_lock.lock().await)
    }

    pub fn Get_current_screen(&self) -> Result_type<*mut LVGL::lv_obj_t> {
        Ok(unsafe { LVGL::lv_screen_active() })
    }
}
