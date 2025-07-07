use alloc::string::String;
use core::ffi::CStr;
use File_system::Size_type;
use Graphics::{Color_type, Event_code_type, Key_type, Window_type, LVGL};
use Synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use crate::Error::Result_type;

pub(crate) struct Inner_type {
    window: Window_type,
    buffer: String,
    display: *mut LVGL::lv_obj_t,
    input: *mut LVGL::lv_obj_t,
    validated: Option<(&'static str, usize)>,
}

pub struct Terminal_type(pub(crate) RwLock<CriticalSectionRawMutex, Inner_type>);

unsafe impl Send for Terminal_type {}

unsafe impl Sync for Terminal_type {}

impl Terminal_type {
    const CLEAR: &'static str = "\x1B[2J";
    const HOME: &'static str = "\x1B[H";

    pub async fn New() -> Result_type<Self> {
        let _lock = Graphics::Get_instance().Lock().await;

        let mut Window = Graphics::Get_instance().Create_window().await?;

        unsafe {
            Window.Set_icon(">_", Color_type::BLACK);

            LVGL::lv_obj_set_flex_flow(
                Window.Get_object(),
                LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
        }

        let Container = unsafe {
            let container = LVGL::lv_obj_create(Window.Get_object());

            LVGL::lv_obj_set_width(container, LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_grow(container, 1);
            LVGL::lv_obj_set_scroll_snap_y(container, LVGL::lv_scroll_snap_t_LV_SCROLL_SNAP_END);

            container
        };

        let Buffer = String::with_capacity(80 * 24);

        let Display = unsafe {
            let label = LVGL::lv_label_create(Container);

            if label.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_width(label, LVGL::lv_pct(100));
            LVGL::lv_label_set_text_static(label, Buffer.as_ptr() as *const i8);
            LVGL::lv_obj_set_style_text_font(
                label,
                &raw const LVGL::lv_font_unscii_8,
                LVGL::LV_STATE_DEFAULT,
            );

            label
        };

        let Input = unsafe {
            let input = LVGL::lv_textarea_create(Window.Get_object());

            if input.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            LVGL::lv_textarea_set_placeholder_text(input, c"Enter your command ...".as_ptr());
            LVGL::lv_obj_set_width(input, LVGL::lv_pct(100));
            LVGL::lv_textarea_set_one_line(input, true);

            input
        };

        let Inner = Inner_type {
            window: Window,
            buffer: Buffer,
            display: Display,
            input: Input,
            validated: None,
        };

        Ok(Self(RwLock::new(Inner)))
    }

    pub async fn Print(&self, Text: &str) -> Result_type<()> {
        let mut inner = self.0.write().await;

        Self::Print_internal(&mut inner, Text).await?;

        Ok(())
    }

    async fn Print_internal(Inner: &mut Inner_type, Text: &str) -> Result_type<()> {
        if !Inner.buffer.is_empty() {
            let last_index = Inner.buffer.len() - 1;

            Inner.buffer.remove(last_index);
        }

        let Start_index = if let Some(Last_clear) = Text.rfind(Self::CLEAR) {
            Inner.buffer.clear();
            Last_clear + Self::CLEAR.len()
        } else {
            0
        };

        let Start_index = if let Some(Last_home) = Text.rfind(Self::HOME) {
            Inner.buffer.clear();
            Last_home + Self::HOME.len()
        } else {
            Start_index
        };

        Inner.buffer += &Text[Start_index..];
        Inner.buffer += "\0";

        let _Lock = Graphics::Get_instance().Lock().await;

        unsafe {
            LVGL::lv_label_set_text(Inner.display, Inner.buffer.as_ptr() as *const i8);
            LVGL::lv_obj_scroll_to_view(Inner.display, true);
        }

        Ok(())
    }

    async fn Print_line_internal(Inner: &mut Inner_type, Text: &str) -> Result_type<()> {
        if !Inner.buffer.is_empty() {
            let last_index = Inner.buffer.len() - 1;

            Inner.buffer.remove(last_index);
        }

        let Start_index = if let Some(Last_clear) = Text.rfind("\x1B[2J") {
            Inner.buffer.clear();
            Last_clear + 4
        } else {
            0
        };

        Inner.buffer += Text[Start_index..].trim();
        Inner.buffer += "\n\0";

        let _Lock = Graphics::Get_instance().Lock().await;

        unsafe {
            LVGL::lv_label_set_text(Inner.display, Inner.buffer.as_ptr() as *const i8);
            LVGL::lv_obj_scroll_to_view(Inner.display, true);
        }

        Ok(())
    }

    pub async fn Read_input(&self, Buffer: &mut [u8]) -> Result_type<Size_type> {
        let mut inner = self.0.write().await;

        let (String, Index) = match inner.validated.as_mut() {
            Some(validated) => validated,
            None => return Ok(Size_type::New(0)),
        };

        if *Index >= String.len() {
            let _lock = Graphics::Get_instance().Lock().await;

            unsafe {
                LVGL::lv_textarea_set_text(inner.input, c"".as_ptr());
                LVGL::lv_obj_remove_state(inner.input, LVGL::LV_STATE_DISABLED as _);
            }

            inner.validated.take();

            if let Some(Byte) = Buffer.first_mut() {
                *Byte = b'\n';
            }

            return Ok(Size_type::New(1));
        }

        let mut Read = 0;

        Buffer
            .iter_mut()
            .zip(&String.as_bytes()[*Index..])
            .for_each(|(byte, &char)| {
                *byte = char;
                *Index += 1;
                Read += 1;
            });

        Ok(Size_type::New(Read))
    }

    pub async fn Event_handler(&self) -> Result_type<bool> {
        let mut inner = self.0.write().await;

        while let Some(Event) = inner.window.Pop_event() {
            match Event.Get_code() {
                Event_code_type::Delete => return Ok(false),
                Event_code_type::Key => {
                    if let Some(Key_type::Character(Character)) = Event.Get_key() {
                        if inner.validated.is_some() {
                            continue;
                        }

                        if Character == b'\n' || Character == b'\r' {
                            let _lock = Graphics::Get_instance().Lock().await;

                            let Text = unsafe {
                                let text = LVGL::lv_textarea_get_text(inner.input);

                                CStr::from_ptr(text).to_str()?
                            };

                            unsafe {
                                LVGL::lv_obj_add_state(inner.input, LVGL::LV_STATE_DISABLED as _);
                            }

                            drop(_lock);

                            Self::Print_line_internal(&mut inner, Text).await?;

                            inner.validated.replace((Text, 0));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }
}
