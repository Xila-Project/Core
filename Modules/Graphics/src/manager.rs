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
use file_system::Device_type;

use super::lvgl;

use super::Point_type;

use crate::window::Window_type;
use crate::Color_type;
use crate::Display_type;
use crate::Input_type;
use crate::Input_type_type;
use crate::{Error_type, Result_type, Screen_read_data_type};

static MANAGER_INSTANCE: OnceLock<Manager_type> = OnceLock::new();

pub async fn initialize(
    screen_device: Device_type,
    input_device: Device_type,
    input_device_type: Input_type_type,
    buffer_size: usize,
    double_buffered: bool,
) -> &'static Manager_type {
    let manager = Manager_type::new(
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

pub fn get_instance() -> &'static Manager_type {
    MANAGER_INSTANCE
        .try_get()
        .expect("Graphics manager not initialized")
}

struct Inner_type {
    _inputs: Vec<Input_type>,
    _displays: Vec<Display_type>,
    window_parent: *mut lvgl::lv_obj_t,
}

pub struct Manager_type {
    inner: RwLock<CriticalSectionRawMutex, Inner_type>,
    global_lock: Mutex<CriticalSectionRawMutex, ()>,
}

impl Drop for Manager_type {
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
        .as_milliseconds() as u32
}

unsafe impl Send for Manager_type {}

unsafe impl Sync for Manager_type {}

impl Manager_type {
    fn new(
        _: &time::Manager_type,
        screen_device: Device_type,
        input_device: Device_type,
        input_device_type: Input_type_type,
        buffer_size: usize,
        double_buffered: bool,
    ) -> Result_type<Self> {
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
            inner: RwLock::new(Inner_type {
                _inputs: vec![input],
                _displays: vec![display],

                window_parent: screen,
            }),
            global_lock: Mutex::new(()),
        })
    }

    pub async fn r#loop<F>(&self, sleep: impl Fn(Duration) -> F + Send + 'static) -> Result_type<()>
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

    pub async fn set_window_parent(&self, window_parent: *mut lvgl::lv_obj_t) -> Result_type<()> {
        self.inner.write().await.window_parent = window_parent;

        Ok(())
    }

    pub async fn create_window(&self) -> Result_type<Window_type> {
        let parent_object = self.inner.write().await.window_parent;

        let window = unsafe { Window_type::new(parent_object)? };

        Ok(window)
    }

    pub async fn add_input_device(
        &self,

        input_device: Device_type,
        input_type: Input_type_type,
    ) -> Result_type<()> {
        let input = Input_type::new(input_device, input_type)?;

        self.inner.write().await._inputs.push(input);

        Ok(())
    }

    fn create_display(
        screen_device: Device_type,
        buffer_size: usize,
        input_device: Device_type,
        input_device_type: Input_type_type,
        double_buffered: bool,
    ) -> Result_type<(Display_type, Input_type)> {
        let mut screen_read_data = Screen_read_data_type::default();

        screen_device
            .read(screen_read_data.as_mut())
            .map_err(|_| Error_type::Failed_to_get_resolution)?;

        let resolution: Point_type = screen_read_data.get_resolution();

        let display = Display_type::new(screen_device, resolution, buffer_size, double_buffered)?;

        let input = Input_type::new(input_device, input_device_type)?;

        Ok((display, input))
    }

    pub async fn get_window_count(&self) -> Result_type<usize> {
        let window_parent = self.inner.read().await.window_parent;
        unsafe { Ok(lvgl::lv_obj_get_child_count(window_parent) as usize) }
    }

    pub async fn get_window_icon(&self, index: usize) -> Result_type<(String, Color_type)> {
        let window_parent = self.inner.read().await.window_parent;

        let window = unsafe {
            let child = lvgl::lv_obj_get_child(window_parent, index as i32);

            Window_type::from_raw(child)
        };

        let icon = window.get_icon();

        let icon = (icon.0.to_string(), icon.1);

        forget(window);

        Ok(icon)
    }

    pub async fn get_window_identifier(&self, index: usize) -> Result_type<usize> {
        let window_parent = self.inner.read().await.window_parent;

        let window = unsafe { lvgl::lv_obj_get_child(window_parent, index as i32) as usize };

        Ok(window)
    }

    pub async fn maximize_window(&self, identifier: usize) -> Result_type<()> {
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
            Err(Error_type::Invalid_window_identifier)
        }
    }

    pub async fn lock_function<T>(
        &self,
        function: impl FnOnce() -> Result_type<T>,
    ) -> Result_type<T> {
        let _lock = self.global_lock.lock().await;

        function()
    }

    pub async fn lock(&self) -> MutexGuard<'_, CriticalSectionRawMutex, ()> {
        self.global_lock.lock().await
    }

    pub fn get_current_screen(&self) -> Result_type<*mut lvgl::lv_obj_t> {
        Ok(unsafe { lvgl::lv_screen_active() })
    }
}
