use File_system::Status_type;
use Graphics::{Window_type, LVGL};

use crate::Error::Result_type;

pub struct Terminal_type {
    Window: Window_type,
    Text_area: *mut LVGL::lv_obj_t,
}

impl Terminal_type {
    pub fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock()?; // Lock the graphics

        let Window = Graphics::Get_instance().Create_window()?;

        let Text_area = unsafe {
            let Text_area = LVGL::lv_textarea_create(Window.Get_object());

            if Text_area.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            Text_area
        };

        let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

        //let (Input, Output) = Virtual_file_system::Get_instance().Create_unnamed_pipe(Task, Status_type::None, 32)?;

        //Executable::Execute("/Executables/Shell_command_line", "".to_string(), false);

        Ok(Self { Window, Text_area })
    }

    pub fn Event_handler(&mut self) {
        while let Some(Event) = self.Window.Pop_event() {}
    }
}
