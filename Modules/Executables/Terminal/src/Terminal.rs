use std::{ffi::CStr, sync::RwLock};
use Graphics::{Event_code_type, Key_type, Window_type, LVGL};

use crate::Error::Result_type;

pub(crate) struct Inner_type {
    Window: Window_type,
    Buffer: String,
    Display: *mut LVGL::lv_obj_t,
    Input: *mut LVGL::lv_obj_t,
    Validated: bool,
}

pub struct Terminal_type(pub(crate) RwLock<Inner_type>);

unsafe impl Send for Terminal_type {}

unsafe impl Sync for Terminal_type {}

impl Terminal_type {
    pub fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock()?;

        let Window = Graphics::Get_instance().Create_window()?;

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

        let Inner = Inner_type {
            Window,
            Buffer,
            Display,
            Input,
            Validated: false,
        };

        Ok(Self(RwLock::new(Inner)))
    }

    pub fn Print(&self, Text: &str) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        Self::Print_internal(&mut Inner, Text)?;

        Ok(())
    }

    fn Print_internal(Inner: &mut Inner_type, Text: &str) -> Result_type<()> {
        if !Inner.Buffer.is_empty() {
            let Last_index = Inner.Buffer.len() - 1;

            Inner.Buffer.remove(Last_index);
        }

        Inner.Buffer += Text;
        Inner.Buffer += "\0";

        let _Lock = Graphics::Get_instance().Lock().unwrap();

        unsafe {
            LVGL::lv_label_set_text(Inner.Display, Inner.Buffer.as_ptr() as *const i8);
        }

        Ok(())
    }

    pub fn Read_input(&self, String: &mut [u8]) -> Result_type<usize> {
        let mut Inner = self.0.write()?;

        if !Inner.Validated {
            return Ok(0);
        }

        let _Lock = Graphics::Get_instance().Lock()?;

        let Text = unsafe {
            LVGL::lv_textarea_add_char(Inner.Input, '\n' as u32);
            let Text = LVGL::lv_textarea_get_text(Inner.Input);

            CStr::from_ptr(Text).to_str()?
        };

        let Length = Text.len().min(String.len());

        String[..Length].copy_from_slice(&Text.as_bytes()[..Length]);

        unsafe {
            LVGL::lv_textarea_set_text(Inner.Input, c"".as_ptr());
        }

        Inner.Validated = false;

        Ok(Length)
    }

    pub fn Event_handler(&self) -> Result_type<bool> {
        let mut Inner = self.0.write()?;

        while let Some(Event) = Inner.Window.Pop_event() {
            match Event.Get_code() {
                Event_code_type::Delete => return Ok(false),
                Event_code_type::Key => {
                    if let Some(Key_type::Character(Character)) = Event.Get_key() {
                        if Inner.Validated {
                            continue;
                        }

                        if Character == b'\n' || Character == b'\r' {
                            let _Lock = Graphics::Get_instance().Lock()?;

                            let Text = unsafe {
                                let Text = LVGL::lv_textarea_get_text(Inner.Input);

                                CStr::from_ptr(Text).to_str()?
                            };

                            drop(_Lock);

                            Self::Print_internal(&mut Inner, Text)?;

                            Inner.Validated = true;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }
}
