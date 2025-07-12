use alloc::string::String;

use core::ffi::CStr;
use file_system::Size;
use graphics::{lvgl, Color, EventKind, Key, Window};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

use crate::error::Result;

pub(crate) struct Inner {
    window: Window,
    buffer: String,
    display: *mut lvgl::lv_obj_t,
    input: *mut lvgl::lv_obj_t,
    validated: Option<(&'static str, usize)>,
}

pub struct Terminal(pub(crate) RwLock<CriticalSectionRawMutex, Inner>);

unsafe impl Send for Terminal {}

unsafe impl Sync for Terminal {}

impl Terminal {
    const CLEAR: &'static str = "\x1B[2J";
    const HOME: &'static str = "\x1B[H";

    pub async fn new() -> Result<Self> {
        let _lock = graphics::get_instance().lock().await;

        let mut window = graphics::get_instance().create_window().await?;

        unsafe {
            window.set_icon(">_", Color::BLACK);

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
                return Err(crate::error::Error::FailedToCreateObject);
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
                return Err(crate::error::Error::FailedToCreateObject);
            }

            lvgl::lv_textarea_set_placeholder_text(input, c"Enter your command ...".as_ptr());
            lvgl::lv_obj_set_width(input, lvgl::lv_pct(100));
            lvgl::lv_textarea_set_one_line(input, true);

            input
        };

        let inner = Inner {
            window,
            buffer,
            display,
            input,
            validated: None,
        };

        Ok(Self(RwLock::new(inner)))
    }

    pub async fn print(&self, text: &str) -> Result<()> {
        let mut inner = self.0.write().await;

        Self::print_internal(&mut inner, text).await?;

        Ok(())
    }

    async fn print_internal(inner: &mut Inner, text: &str) -> Result<()> {
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

    async fn print_line_internal(inner: &mut Inner, text: &str) -> Result<()> {
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

    pub async fn read_input(&self, buffer: &mut [u8]) -> Result<Size> {
        let mut inner = self.0.write().await;

        let (string, index) = match inner.validated.as_mut() {
            Some(validated) => validated,
            None => return Ok(Size::new(0)),
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

            return Ok(Size::new(1));
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

        Ok(Size::new(read))
    }

    pub async fn event_handler(&self) -> Result<bool> {
        let mut inner = self.0.write().await;

        while let Some(event) = inner.window.pop_event() {
            match event.get_code() {
                EventKind::Delete => return Ok(false),
                EventKind::Key => {
                    if let Some(Key::Character(character)) = event.get_key() {
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
