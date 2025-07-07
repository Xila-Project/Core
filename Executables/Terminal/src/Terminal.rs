use alloc::string::String;
use core::ffi::CStr;
use File_system::Size_type;
use Graphics::{Color_type, Event_code_type, Key_type, Window_type, LVGL};
use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use crate::Error::Result_type;

pub(crate) struct Inner_type {
    Window: Window_type,
    Buffer: String,
    Display: *mut LVGL::lv_obj_t,
    Input: *mut LVGL::lv_obj_t,
    Validated: Option<(&'static str, usize)>,
}

pub struct Terminal_type(pub(crate) RwLock<CriticalSectionRawMutex, Inner_type>);

unsafe impl Send for Terminal_type {}

unsafe impl Sync for Terminal_type {}

impl Terminal_type {
    const CLEAR: &'static str = "\x1B[2J";
    const HOME: &'static str = "\x1B[H";

    pub async fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock().await;

        let mut Window = Graphics::Get_instance().Create_window().await?;

        unsafe {
            Window.Set_icon(">_", Color_type::BLACK);

            LVGL::lv_obj_set_flex_flow(
                Window.Get_object(),
                LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
        }

        let Container = unsafe {
            let Container = LVGL::lv_obj_create(Window.Get_object());

            LVGL::lv_obj_set_width(Container, LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_grow(Container, 1);
            LVGL::lv_obj_set_scroll_snap_y(Container, LVGL::lv_scroll_snap_t_LV_SCROLL_SNAP_END);

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
            Validated: None,
        };

        Ok(Self(RwLock::new(Inner)))
    }

    pub async fn Print(&self, Text: &str) -> Result_type<()> {
        let mut Inner = self.0.write().await;

        Self::Print_internal(&mut Inner, Text).await?;

        Ok(())
    }

    async fn Print_internal(Inner: &mut Inner_type, Text: &str) -> Result_type<()> {
        if !Inner.Buffer.is_empty() {
            let Last_index = Inner.Buffer.len() - 1;

            Inner.Buffer.remove(Last_index);
        }

        let Start_index = if let Some(Last_clear) = Text.rfind(Self::CLEAR) {
            Inner.Buffer.clear();
            Last_clear + Self::CLEAR.len()
        } else {
            0
        };

        let Start_index = if let Some(Last_home) = Text.rfind(Self::HOME) {
            Inner.Buffer.clear();
            Last_home + Self::HOME.len()
        } else {
            Start_index
        };

        Inner.Buffer += &Text[Start_index..];
        Inner.Buffer += "\0";

        let _Lock = Graphics::Get_instance().Lock().await;

        unsafe {
            LVGL::lv_label_set_text(Inner.Display, Inner.Buffer.as_ptr() as *const i8);
            LVGL::lv_obj_scroll_to_view(Inner.Display, true);
        }

        Ok(())
    }

    async fn Print_line_internal(Inner: &mut Inner_type, Text: &str) -> Result_type<()> {
        if !Inner.Buffer.is_empty() {
            let Last_index = Inner.Buffer.len() - 1;

            Inner.Buffer.remove(Last_index);
        }

        let Start_index = if let Some(Last_clear) = Text.rfind("\x1B[2J") {
            Inner.Buffer.clear();
            Last_clear + 4
        } else {
            0
        };

        Inner.Buffer += Text[Start_index..].trim();
        Inner.Buffer += "\n\0";

        let _Lock = Graphics::Get_instance().Lock().await;

        unsafe {
            LVGL::lv_label_set_text(Inner.Display, Inner.Buffer.as_ptr() as *const i8);
            LVGL::lv_obj_scroll_to_view(Inner.Display, true);
        }

        Ok(())
    }

    pub async fn Read_input(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        let mut Inner = self.0.write().await;

        let (String, Index) = match Inner.Validated.as_mut() {
            Some(Validated) => Validated,
            None => return Ok(Size_type::New(0)),
        };

        if *Index >= String.len() {
            let _Lock = Graphics::Get_instance().Lock().await;

            unsafe {
                LVGL::lv_textarea_set_text(Inner.Input, c"".as_ptr());
                LVGL::lv_obj_remove_state(Inner.Input, LVGL::LV_STATE_DISABLED as _);
            }

            Inner.Validated.take();

            if let Some(Byte) = Buffer.first_mut() {
                *Byte = b'\n';
            }

            return Ok(Size_type::New(1));
        }

        let mut Read = 0;

        Buffer
            .iter_mut()
            .zip(&String.as_bytes()[*Index..])
            .for_each(|(Byte, &Char)| {
                *Byte = Char;
                *Index += 1;
                Read += 1;
            });

        Ok(Size_type::New(Read))
    }

    pub async fn Event_handler(&self) -> Result_type<bool> {
        let mut Inner = self.0.write().await;

        while let Some(Event) = Inner.Window.Pop_event() {
            match Event.Get_code() {
                Event_code_type::Delete => return Ok(false),
                Event_code_type::Key => {
                    if let Some(Key_type::Character(Character)) = Event.Get_key() {
                        if Inner.Validated.is_some() {
                            continue;
                        }

                        if Character == b'\n' || Character == b'\r' {
                            let _Lock = Graphics::Get_instance().Lock().await;

                            let Text = unsafe {
                                let Text = LVGL::lv_textarea_get_text(Inner.Input);

                                CStr::from_ptr(Text).to_str()?
                            };

                            unsafe {
                                LVGL::lv_obj_add_state(Inner.Input, LVGL::LV_STATE_DISABLED as _);
                            }

                            drop(_Lock);

                            Self::Print_line_internal(&mut Inner, Text).await?;

                            Inner.Validated.replace((Text, 0));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }
}
