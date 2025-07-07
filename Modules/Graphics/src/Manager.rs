use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use Synchronization::blocking_mutex::raw::CriticalSectionRawMutex;
use Synchronization::mutex::{Mutex, MutexGuard};
use Synchronization::{once_lock::OnceLock, rwlock::RwLock};

use core::{future::Future, mem::forget};

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

static MANAGER_INSTANCE: OnceLock<Manager_type> = OnceLock::new();

pub async fn Initialize(
    screen_device: Device_type,
    input_device: Device_type,
    input_device_type: Input_type_type,
    buffer_size: usize,
    double_buffered: bool,
) -> &'static Manager_type {
    let Manager = Manager_type::new(
        Time::Get_instance(),
        screen_device,
        input_device,
        input_device_type,
        buffer_size,
        double_buffered,
    )
    .expect("Failed to create manager instance");

    MANAGER_INSTANCE.get_or_init(|| Manager)
}

pub fn Get_instance() -> &'static Manager_type {
    MANAGER_INSTANCE
        .try_get()
        .expect("Graphics manager not initialized")
}

struct Inner_type {
    _inputs: Vec<Input_type>,
    _displays: Vec<Display_type>,
    window_parent: *mut LVGL::lv_obj_t,
}

pub struct Manager_type {
    inner: RwLock<CriticalSectionRawMutex, Inner_type>,
    global_lock: Mutex<CriticalSectionRawMutex, ()>,
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
    fn new(
        _: &Time::Manager_type,
        screen_device: Device_type,
        input_device: Device_type,
        input_device_type: Input_type_type,
        buffer_size: usize,
        double_buffered: bool,
    ) -> Result_type<Self> {
        unsafe {
            LVGL::lv_init();

            if !LVGL::lv_is_initialized() {
                panic!("Failed to initialize lvgl");
            }

            LVGL::lv_tick_set_cb(Some(Binding_tick_callback_function));
        }

        let (Display, Input) = Self::Create_display(
            screen_device,
            buffer_size,
            input_device,
            input_device_type,
            double_buffered,
        )?;

        let Screen = Display.Get_object();

        unsafe {
            let group = LVGL::lv_group_create();
            LVGL::lv_group_set_default(group);
        }

        Ok(Self {
            inner: RwLock::new(Inner_type {
                _inputs: vec![Input],
                _displays: vec![Display],

                window_parent: Screen,
            }),
            global_lock: Mutex::new(()),
        })
    }

    pub async fn Loop<F>(&self, Sleep: impl Fn(Duration) -> F + Send + 'static) -> Result_type<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        loop {
            let Time_until_next = unsafe {
                let _lock = self.global_lock.lock().await;
                LVGL::lv_timer_handler()
            };

            Sleep(Duration::from_millis(Time_until_next as u64)).await;
        }
    }

    pub async fn Set_window_parent(&self, Window_parent: *mut LVGL::lv_obj_t) -> Result_type<()> {
        self.inner.write().await.window_parent = Window_parent;

        Ok(())
    }

    pub async fn Create_window(&self) -> Result_type<Window_type> {
        let parent_object = self.inner.write().await.window_parent;

        let Window = unsafe { Window_type::New(parent_object)? };

        Ok(Window)
    }

    pub async fn Add_input_device(
        &self,
        input_device: Device_type,
        input_type: Input_type_type,
    ) -> Result_type<()> {
        let input = Input_type::new(input_device, input_type)?;

        self.inner.write().await._inputs.push(input);

        Ok(())
    }

    fn Create_display(
        screen_device: Device_type,
        buffer_size: usize,
        input_device: Device_type,
        input_device_type: Input_type_type,
        double_buffered: bool,
    ) -> Result_type<(Display_type, Input_type)> {
        let mut screen_read_data = Screen_read_data_type::default();

        screen_device
            .Read(screen_read_data.as_mut())
            .map_err(|_| Error_type::Failed_to_get_resolution)?;

        let Resolution: Point_type = screen_read_data.get_resolution();

        let Display = Display_type::new(screen_device, Resolution, buffer_size, double_buffered)?;

        let Input = Input_type::new(input_device, input_device_type)?;

        Ok((Display, Input))
    }

    pub async fn Get_window_count(&self) -> Result_type<usize> {
        let window_parent = self.inner.read().await.window_parent;
        unsafe { Ok(LVGL::lv_obj_get_child_count(window_parent) as usize) }
    }

    pub async fn Get_window_icon(&self, Index: usize) -> Result_type<(String, Color_type)> {
        let window_parent = self.inner.read().await.window_parent;

        let Window = unsafe {
            let child = LVGL::lv_obj_get_child(window_parent, Index as i32);

            Window_type::From_raw(child)
        };

        let Icon = Window.Get_icon();

        let Icon = (Icon.0.to_string(), Icon.1);

        forget(Window);

        Ok(Icon)
    }

    pub async fn Get_window_identifier(&self, Index: usize) -> Result_type<usize> {
        let window_parent = self.inner.read().await.window_parent;

        let Window = unsafe { LVGL::lv_obj_get_child(window_parent, Index as i32) as usize };

        Ok(Window)
    }

    pub async fn Maximize_window(&self, Identifier: usize) -> Result_type<()> {
        let window_count = self.Get_window_count().await?;

        let Window_parent = self.inner.read().await.window_parent;

        let Found = (0..window_count).find(|Index| unsafe {
            let child = LVGL::lv_obj_get_child(Window_parent, *Index as i32);

            child == Identifier as *mut LVGL::lv_obj_t
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

    pub async fn Lock_function<T>(
        &self,
        function: impl FnOnce() -> Result_type<T>,
    ) -> Result_type<T> {
        let _lock = self.global_lock.lock().await;

        function()
    }

    pub async fn Lock(&self) -> MutexGuard<'_, CriticalSectionRawMutex, ()> {
        self.global_lock.lock().await
    }

    pub fn Get_current_screen(&self) -> Result_type<*mut LVGL::lv_obj_t> {
        Ok(unsafe { LVGL::lv_screen_active() })
    }
}
