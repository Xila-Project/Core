#![allow(non_camel_case_types)]

extern crate alloc;

#[cfg(target_os = "linux")]
use task::Test;

#[cfg(target_os = "linux")]
#[Test]
#[ignore]
async fn main() {
    use std::{process::exit, ptr::null_mut};

    use drivers::native::window_screen;
    use file_system::create_device;
    use graphics::{lvgl, Get_recommended_buffer_size, Input_type_type, Point_type};

    let _ = users::initialize();

    let task_instance = task::initialize();

    time::initialize(create_device!(drivers::native::Time_driver_type::new()))
        .expect("Error initializing time manager");

    const RESOLUTION: Point_type = Point_type::new(800, 480);

    const BUFFER_SIZE: usize = get_recommended_buffer_size(&RESOLUTION);

    let (screen_device, pointer_device, keyboard_device) =
        window_screen::new(RESOLUTION).expect("Error creating touchscreen");

    let _task = task_instance.get_current_task_identifier().await;

    let graphics = graphics::initialize(
        screen_device,
        pointer_device,
        Input_type_type::Pointer,
        BUFFER_SIZE,
        true,
    )
    .await;

    graphics
        .add_input_device(keyboard_device, Input_type_type::Keypad)
        .await
        .unwrap();

    let window = graphics.create_window().await.unwrap();

    let window = window.into_raw();

    let _calendar = unsafe { lvgl::lv_calendar_create(window) };

    let button = unsafe { lvgl::lv_button_create(window) };

    let label = unsafe { lvgl::lv_label_create(button) };

    let slider = unsafe { lvgl::lv_slider_create(window) };

    unsafe extern "C" fn quit(_event: *mut lvgl::lv_event_t) {
        exit(0);
    }

    unsafe {
        lvgl::lv_obj_set_align(slider, lvgl::lv_align_t_LV_ALIGN_LEFT_MID);
        lvgl::lv_obj_set_align(button, lvgl::lv_align_t_LV_ALIGN_CENTER);
        lvgl::lv_label_set_text(label, c"Quit".as_ptr());
        lvgl::lv_obj_add_event_cb(
            button,
            Some(quit),
            lvgl::lv_event_code_t_LV_EVENT_CLICKED,
            null_mut(),
        );
    }

    graphics.r#loop(task::Manager_type::sleep).await.unwrap();
}
