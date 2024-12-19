use core::num::NonZeroUsize;
use std::ffi::CStr;

use Executable::Standard_type;
use Graphics::{
    Event_code_type, Key_type, Window_type,
    LVGL::{self, lv_indev_active},
};

use crate::Error::Result_type;

pub struct Terminal_type {
    Window: Window_type,
    Running: bool,
    Buffer: String,
    Display: *mut LVGL::lv_obj_t,
    Input: *mut LVGL::lv_obj_t,
}

impl Terminal_type {
    pub fn New() -> Result_type<Self> {
        let Window = Graphics::Get_instance().Create_window()?;

        let _Lock = Graphics::Get_instance().Lock()?;

        unsafe {
            LVGL::lv_obj_set_flex_flow(
                Window.Get_object(),
                LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
        }

        let Container = unsafe {
            let Container = LVGL::lv_obj_create(Window.Get_object());

            LVGL::lv_obj_set_width(Container, LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_grow(Container, 1);

            Container
        };

        let Buffer = String::with_capacity(80 * 24);

        let Display = unsafe {
            let Label = LVGL::lv_label_create(Container);

            if Label.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_width(Label, LVGL::lv_pct(100));
            LVGL::lv_label_set_text_static(Label, Buffer.as_ptr() as *const i8);
            LVGL::lv_obj_set_style_text_font(
                Label,
                &raw const LVGL::lv_font_unscii_8,
                LVGL::LV_STATE_DEFAULT,
            );

            Label
        };

        let Input = unsafe {
            let Input = LVGL::lv_textarea_create(Window.Get_object());

            if Input.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            LVGL::lv_textarea_set_placeholder_text(Input, c"Enter your command ...".as_ptr());
            LVGL::lv_obj_set_width(Input, LVGL::lv_pct(100));
            LVGL::lv_textarea_set_one_line(Input, true);

            Input
        };

        Ok(Self {
            Buffer,
            Window,
            Running: true,
            Display,
            Input,
        })
    }

    pub fn Event_handler(&mut self) {
        while let Some(Event) = self.Window.Pop_event() {
            match Event.Get_code() {
                Event_code_type::Delete => self.Running = false,
                Event_code_type::Key => {
                    if let Some(Key_type::Character(Character)) = Event.Get_key() {
                        if Character == b'\n' || Character == b'\r' {
                            unsafe {
                                let Text = LVGL::lv_textarea_get_text(self.Input);

                                if !self.Buffer.is_empty() {
                                    self.Buffer.remove(self.Buffer.len() - 1);
                                }

                                if let Ok(Text) = CStr::from_ptr(Text).to_str() {
                                    self.Buffer += Text;
                                    self.Buffer += "\n\0";
                                    LVGL::lv_label_set_text(
                                        self.Display,
                                        self.Buffer.as_ptr() as *const i8,
                                    );
                                }

                                LVGL::lv_textarea_set_text(self.Input, c"".as_ptr());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn Main(&mut self, _: String) {
        while self.Running {
            self.Event_handler();
        }
    }
}

pub fn Main(Standard: Standard_type, Arguments: String) -> Result<(), NonZeroUsize> {
    Terminal_type::New()
        .map_err(|Error| {
            Standard.Print_error(&Error.to_string());
            NonZeroUsize::from(Error)
        })?
        .Main(Arguments);

    Ok(())
}
