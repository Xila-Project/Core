#![allow(non_camel_case_types)]

extern crate alloc;

#[cfg(target_os = "linux")]
use task::Test;

#[cfg(target_os = "linux")]
#[Test]
#[ignore]
async fn main() {
    use std::{process::exit, ptr::null_mut};

    use drivers::Native::Window_screen;
    use file_system::Create_device;
    use graphics::{Get_recommended_buffer_size, Input_type_type, Point_type, LVGL};

    let _ = Users::Initialize();

    let task_instance = Task::Initialize();

    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::new()))
        .expect("Error initializing time manager");

    const RESOLUTION: Point_type = Point_type::new(800, 480);

    const BUFFER_SIZE: usize = get_recommended_buffer_size(&RESOLUTION);

    let (screen_device, pointer_device, keyboard_device) =
        Window_screen::New(RESOLUTION).expect("Error creating touchscreen");

    let _task = task_instance.get_current_task_identifier().await;

    let graphics = Graphics::initialize(
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

    let _calendar = unsafe { LVGL::lv_calendar_create(window) };

    let button = unsafe { LVGL::lv_button_create(window) };

    let label = unsafe { LVGL::lv_label_create(button) };

    let slider = unsafe { LVGL::lv_slider_create(window) };

    unsafe extern "C" fn quit(_event: *mut LVGL::lv_event_t) {
        exit(0);
    }

    unsafe {
        LVGL::lv_obj_set_align(slider, LVGL::lv_align_t_LV_ALIGN_LEFT_MID);
        LVGL::lv_obj_set_align(button, LVGL::lv_align_t_LV_ALIGN_CENTER);
        LVGL::lv_label_set_text(label, c"Quit".as_ptr());
        LVGL::lv_obj_add_event_cb(
            button,
            Some(quit),
            LVGL::lv_event_code_t_LV_EVENT_CLICKED,
            null_mut(),
        );
    }

    graphics.r#loop(Task::Manager_type::Sleep).await.unwrap();
}
