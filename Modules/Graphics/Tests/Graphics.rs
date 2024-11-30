#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn main() {
    use Drivers::Native::Window_screen;
    use File_system::Create_device;
    use Graphics::{lvgl, Get_recommended_buffer_size, Point_type};
    use Time::Duration_type;

    let _ = Users::Initialize();

    let Task_instance = Task::Initialize().expect("Error initializing task manager");

    Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()))
        .expect("Error initializing time manager");

    const Resolution: Point_type = Point_type::New(800, 480);

    const Buffer_size: usize = Get_recommended_buffer_size(&Resolution);

    let (Screen_device, Pointer_device) =
        Window_screen::New(Resolution).expect("Error creating touchscreen");

    let _Task = Task_instance
        .Get_current_task_identifier()
        .expect("Failed to get current task identifier");

    Graphics::Initialize();

    let Graphics_manager = Graphics::Get_instance();

    let Display = Graphics_manager
        .Create_display::<Buffer_size>(Screen_device, Pointer_device, false)
        .expect("Error adding screen");

    let Screen_object = Display.Get_object();

    let _Calendar = unsafe { lvgl::lv_calendar_create(Screen_object) };

    let Button = unsafe { lvgl::lv_button_create(Screen_object) };

    let Label = unsafe { lvgl::lv_label_create(Button) };

    let Slider = unsafe { lvgl::lv_slider_create(Screen_object) };

    unsafe {
        lvgl::lv_obj_set_align(Slider, lvgl::lv_align_t_LV_ALIGN_LEFT_MID);
        lvgl::lv_obj_set_align(Button, lvgl::lv_align_t_LV_ALIGN_CENTER);
        lvgl::lv_label_set_text(Label, c"Hello world!".as_ptr());
    }

    loop {
        Task::Manager_type::Sleep(Duration_type::from_millis(1000));
    }
}
