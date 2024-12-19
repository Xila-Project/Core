#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn main() {
    use std::{process::exit, ptr::null_mut};

    use Drivers::Native::Window_screen;
    use File_system::Create_device;
    use Graphics::{Get_recommended_buffer_size, Input_type_type, Point_type, LVGL};
    use Time::Duration_type;

    let _ = Users::Initialize();

    let Task_instance = Task::Initialize().expect("Error initializing task manager");

    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()))
        .expect("Error initializing time manager");

    const Resolution: Point_type = Point_type::New(800, 480);

    const Buffer_size: usize = Get_recommended_buffer_size(&Resolution);

    let (Screen_device, Pointer_device, Keyboard_device) =
        Window_screen::New(Resolution).expect("Error creating touchscreen");

    let _Task = Task_instance
        .Get_current_task_identifier()
        .expect("Failed to get current task identifier");

    Graphics::Initialize(
        Screen_device,
        Pointer_device,
        Input_type_type::Pointer,
        Buffer_size,
        true,
    );

    Graphics::Get_instance()
        .Add_input_device(Keyboard_device, Input_type_type::Keypad)
        .unwrap();

    let Window = Graphics::Get_instance().Create_window().unwrap();

    let Window = Window.Into_raw();

    let _Calendar = unsafe { LVGL::lv_calendar_create(Window) };

    let Button = unsafe { LVGL::lv_button_create(Window) };

    let Label = unsafe { LVGL::lv_label_create(Button) };

    let Slider = unsafe { LVGL::lv_slider_create(Window) };

    unsafe extern "C" fn quit(_Event: *mut LVGL::lv_event_t) {
        exit(0);
    }

    unsafe {
        LVGL::lv_obj_set_align(Slider, LVGL::lv_align_t_LV_ALIGN_LEFT_MID);
        LVGL::lv_obj_set_align(Button, LVGL::lv_align_t_LV_ALIGN_CENTER);
        LVGL::lv_label_set_text(Label, c"Quit".as_ptr());
        LVGL::lv_obj_add_event_cb(
            Button,
            Some(quit),
            LVGL::lv_event_code_t_LV_EVENT_CLICKED,
            null_mut(),
        );
    }

    loop {
        Task::Manager_type::Sleep(Duration_type::from_millis(1000));
    }
}
