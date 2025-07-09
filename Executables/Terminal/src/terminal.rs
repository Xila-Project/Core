use alloc::string::String;

use core::ffi::CStr;
use file_system::Size_type;
use graphics::{lvgl, Color_type, Event_code_type, Key_type, Window_type};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use crate::error::Result_type;

pub(crate) struct Inner_type {
    window: Window_type,
    buffer: String,
    display: *mut lvgl::lv_obj_t,
    input: *mut lvgl::lv_obj_t,
    validated: Option<(&'static str, usize)>,
}

pub struct Terminal_type(pub(crate) RwLock<CriticalSectionRawMutex, Inner_type>);

unsafe impl Send for Terminal_type {}

unsafe impl Sync for Terminal_type {}

impl Terminal_type {
    const CLEAR: &'static str = "\x1B[2J";
    const HOME: &'static str = "\x1B[H";

    pub async fn new() -> Result_type<Self> {
        let _lock = graphics::get_instance().lock().await;

        let mut window = graphics::get_instance().create_window().await?;

        unsafe {
            window.set_icon(">_", Color_type::BLACK);

            lvgl::lv_obj_set_flex_flow(
                window.get_object(),
                lvgl::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
        }

        let container = unsafe {
            let container = lvgl::lv_obj_create(window.get_object());

            lvgl::lv_obj_set_width(container, lvgl::lv_pct(100));
            lvgl::lv_obj_set_flex_grow(container, 1);
            lvgl::lv_obj_set_scroll_snap_y(container, lvgl::lv_scroll_snap_t_LV_SCROLL_SNAP_END);

            container
        };

        let buffer = String::with_capacity(80 * 24);

        let display = unsafe {
            let label = lvgl::lv_label_create(container);

            if label.is_null() {
                return Err(crate::error::Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_set_width(label, lvgl::lv_pct(100));
            lvgl::lv_label_set_text_static(label, buffer.as_ptr() as *const i8);
            lvgl::lv_obj_set_style_text_font(
                label,
                &raw const lvgl::lv_font_unscii_8,
                lvgl::LV_STATE_DEFAULT,
            );

            label
        };

        let input = unsafe {
            let input = lvgl::lv_textarea_create(window.get_object());

            if input.is_null() {
                return Err(crate::error::Error_type::Failed_to_create_object);
            }

            lvgl::lv_textarea_set_placeholder_text(input, c"Enter your command ...".as_ptr());
            lvgl::lv_obj_set_width(input, lvgl::lv_pct(100));
            lvgl::lv_textarea_set_one_line(input, true);

            input
        };

        let inner = Inner_type {
            window,
            buffer,
            display,
            input,
            validated: None,
        };

        Ok(Self(RwLock::new(inner)))
    }

    pub async fn print(&self, text: &str) -> Result_type<()> {
        let mut inner = self.0.write().await;

        Self::print_internal(&mut inner, text).await?;

        Ok(())
    }

    async fn print_internal(inner: &mut Inner_type, text: &str) -> Result_type<()> {
        if !inner.buffer.is_empty() {
            let last_index = inner.buffer.len() - 1;

            inner.buffer.remove(last_index);
        }

        let start_index = if let Some(last_clear) = text.rfind(Self::CLEAR) {
            inner.buffer.clear();
            last_clear + Self::CLEAR.len()
        } else {
            0
        };

        let start_index = if let Some(last_home) = text.rfind(Self::HOME) {
            inner.buffer.clear();
            last_home + Self::HOME.len()
        } else {
            start_index
        };

        inner.buffer += &text[start_index..];
        inner.buffer += "\0";

        let _lock = graphics::get_instance().lock().await;

        unsafe {
            lvgl::lv_label_set_text(inner.display, inner.buffer.as_ptr() as *const i8);
            lvgl::lv_obj_scroll_to_view(inner.display, true);
        }

        Ok(())
    }

    async fn print_line_internal(inner: &mut Inner_type, text: &str) -> Result_type<()> {
        if !inner.buffer.is_empty() {
            let last_index = inner.buffer.len() - 1;

            inner.buffer.remove(last_index);
        }

        let start_index = if let Some(last_clear) = text.rfind("\x1B[2J") {
            inner.buffer.clear();
            last_clear + 4
        } else {
            0
        };

        inner.buffer += text[start_index..].trim();
        inner.buffer += "\n\0";

        let _lock = graphics::get_instance().lock().await;

        unsafe {
            lvgl::lv_label_set_text(inner.display, inner.buffer.as_ptr() as *const i8);
            lvgl::lv_obj_scroll_to_view(inner.display, true);
        }

        Ok(())
    }

    pub async fn read_input(&self, buffer: &mut [u8]) -> Result_type<Size_type> {
        let mut inner = self.0.write().await;

        let (string, index) = match inner.validated.as_mut() {
            Some(validated) => validated,
            None => return Ok(Size_type::new(0)),
        };

        if *index >= string.len() {
            let _lock = graphics::get_instance().lock().await;

            unsafe {
                lvgl::lv_textarea_set_text(inner.input, c"".as_ptr());
                lvgl::lv_obj_remove_state(inner.input, lvgl::LV_STATE_DISABLED as _);
            }

            inner.validated.take();

            if let Some(byte) = buffer.first_mut() {
                *byte = b'\n';
            }

            return Ok(Size_type::new(1));
        }

        let mut read = 0;

        buffer
            .iter_mut()
            .zip(&string.as_bytes()[*index..])
            .for_each(|(byte, &char)| {
                *byte = char;
                *index += 1;
                read += 1;
            });

        Ok(Size_type::new(read))
    }

    pub async fn event_handler(&self) -> Result_type<bool> {
        let mut inner = self.0.write().await;

        while let Some(event) = inner.window.pop_event() {
            match event.get_code() {
                Event_code_type::Delete => return Ok(false),
                Event_code_type::Key => {
                    if let Some(Key_type::Character(character)) = event.get_key() {
                        if inner.validated.is_some() {
                            continue;
                        }

                        if character == b'\n' || character == b'\r' {
                            let _lock = graphics::get_instance().lock().await;

                            let text = unsafe {
                                let text = lvgl::lv_textarea_get_text(inner.input);

                                CStr::from_ptr(text).to_str()?
                            };

                            unsafe {
                                lvgl::lv_obj_add_state(inner.input, lvgl::LV_STATE_DISABLED as _);
                            }

                            drop(_lock);

                            Self::print_line_internal(&mut inner, text).await?;

                            inner.validated.replace((text, 0));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }
}
