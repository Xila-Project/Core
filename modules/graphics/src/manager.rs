use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use synchronization::blocking_mutex::raw::CriticalSectionRawMutex;
use synchronization::mutex::{Mutex, MutexGuard};
use synchronization::{once_lock::OnceLock, rwlock::RwLock};

use core::{future::Future, mem::forget};

use core::time::Duration;
use file_system::DeviceType;

use super::lvgl;

use super::Point;

use crate::window::Window;
use crate::Color;
use crate::DisplayType;
use crate::InputKind;
use crate::InputType;
use crate::{Error, Result, ScreenReadData};

static MANAGER_INSTANCE: OnceLock<Manager> = OnceLock::new();

pub async fn initialize(
    screen_device: DeviceType,
    input_device: DeviceType,
    input_device_type: InputKind,
    buffer_size: usize,
    double_buffered: bool,
) -> &'static Manager {
    let manager = Manager::new(
        time::get_instance(),
        screen_device,
        input_device,
        input_device_type,
        buffer_size,
        double_buffered,
    )
    .expect("Failed to create manager instance");

    MANAGER_INSTANCE.get_or_init(|| manager)
}

pub fn get_instance() -> &'static Manager {
    MANAGER_INSTANCE
        .try_get()
        .expect("Graphics manager not initialized")
}

struct InnerType {
    _inputs: Vec<InputType>,
    _displays: Vec<DisplayType>,
    window_parent: *mut lvgl::lv_obj_t,
}

pub struct Manager {
    inner: RwLock<CriticalSectionRawMutex, InnerType>,
    global_lock: Mutex<CriticalSectionRawMutex, ()>,
}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe {
            lvgl::lv_deinit();
        }
    }
}

extern "C" fn binding_tick_callback_function() -> u32 {
    time::get_instance()
        .get_current_time()
        .unwrap_or_default()
        .as_millis() as u32
}

unsafe impl Send for Manager {}

unsafe impl Sync for Manager {}

impl Manager {
    fn new(
        _: &time::Manager,
        screen_device: DeviceType,
        input_device: DeviceType,
        input_device_type: InputKind,
        buffer_size: usize,
        double_buffered: bool,
    ) -> Result<Self> {
        unsafe {
            lvgl::lv_init();

            if !lvgl::lv_is_initialized() {
                panic!("Failed to initialize lvgl");
            }

            lvgl::lv_tick_set_cb(Some(binding_tick_callback_function));
        }

        let (display, input) = Self::create_display(
            screen_device,
            buffer_size,
            input_device,
            input_device_type,
            double_buffered,
        )?;

        let screen = display.get_object();

        unsafe {
            let group = lvgl::lv_group_create();
            lvgl::lv_group_set_default(group);
        }

        Ok(Self {
            inner: RwLock::new(InnerType {
                _inputs: vec![input],
                _displays: vec![display],

                window_parent: screen,
            }),
            global_lock: Mutex::new(()),
        })
    }

    pub async fn r#loop<F>(&self, sleep: impl Fn(Duration) -> F + Send + 'static) -> Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        loop {
            let time_until_next = unsafe {
                let _lock = self.global_lock.lock().await;
                lvgl::lv_timer_handler()
            };

            sleep(Duration::from_millis(time_until_next as u64)).await;
        }
    }

    pub async fn set_window_parent(&self, window_parent: *mut lvgl::lv_obj_t) -> Result<()> {
        self.inner.write().await.window_parent = window_parent;

        Ok(())
    }

    pub async fn create_window(&self) -> Result<Window> {
        let parent_object = self.inner.write().await.window_parent;

        let window = unsafe { Window::new(parent_object)? };

        Ok(window)
    }

    pub async fn add_input_device(
        &self,

        input_device: DeviceType,
        input_type: InputKind,
    ) -> Result<()> {
        let input = InputType::new(input_device, input_type)?;

        self.inner.write().await._inputs.push(input);

        Ok(())
    }

    fn create_display(
        screen_device: DeviceType,
        buffer_size: usize,
        input_device: DeviceType,
        input_device_type: InputKind,
        double_buffered: bool,
    ) -> Result<(DisplayType, InputType)> {
        let mut screen_read_data = ScreenReadData::default();

        screen_device
            .read(screen_read_data.as_mut())
            .map_err(|_| Error::FailedToGetResolution)?;

        let resolution: Point = screen_read_data.get_resolution();

        let display = DisplayType::new(screen_device, resolution, buffer_size, double_buffered)?;

        let input = InputType::new(input_device, input_device_type)?;

        Ok((display, input))
    }

    pub async fn get_window_count(&self) -> Result<usize> {
        let window_parent = self.inner.read().await.window_parent;
        unsafe { Ok(lvgl::lv_obj_get_child_count(window_parent) as usize) }
    }

    pub async fn get_window_icon(&self, index: usize) -> Result<(String, Color)> {
        let window_parent = self.inner.read().await.window_parent;

        let window = unsafe {
            let child = lvgl::lv_obj_get_child(window_parent, index as i32);

            Window::from_raw(child)
        };

        let icon = window.get_icon();

        let icon = (icon.0.to_string(), icon.1);

        forget(window);

        Ok(icon)
    }

    pub async fn get_window_identifier(&self, index: usize) -> Result<usize> {
        let window_parent = self.inner.read().await.window_parent;

        let window = unsafe { lvgl::lv_obj_get_child(window_parent, index as i32) as usize };

        Ok(window)
    }

    pub async fn maximize_window(&self, identifier: usize) -> Result<()> {
        let window_count = self.get_window_count().await?;

        let window_parent = self.inner.read().await.window_parent;

        let found = (0..window_count).find(|index| unsafe {
            let child = lvgl::lv_obj_get_child(window_parent, *index as i32);

            child == identifier as *mut lvgl::lv_obj_t
        });

        if found.is_some() {
            unsafe {
                lvgl::lv_obj_move_foreground(identifier as *mut lvgl::lv_obj_t);
            }

            Ok(())
        } else {
            Err(Error::InvalidWindowIdentifier)
        }
    }

    pub async fn lock_function<T>(&self, function: impl FnOnce() -> Result<T>) -> Result<T> {
        let _lock = self.global_lock.lock().await;

        function()
    }

    pub async fn lock(&self) -> MutexGuard<'_, CriticalSectionRawMutex, ()> {
        self.global_lock.lock().await
    }

    pub fn get_current_screen(&self) -> Result<*mut lvgl::lv_obj_t> {
        Ok(unsafe { lvgl::lv_screen_active() })
    }
}
