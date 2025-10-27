use crate::error::Result;
use alloc::{collections::vec_deque::VecDeque, string::String};
use core::ffi::CStr;
use xila::file_system::Size;
use xila::graphics::{
    self, Color, EventKind, Key, Window,
    lvgl::{self, lv_obj_t},
};
use xila::synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};

pub(crate) struct Inner {
    window: Window,
    buffer: String,
    display: *mut lvgl::lv_obj_t,
    input: *mut lvgl::lv_obj_t,
    validated_input: VecDeque<u8>,
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

        let mut buffer = String::with_capacity(80 * 24);
        buffer.push('\0'); // Initialize with null terminator for LVGL

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
            validated_input: VecDeque::with_capacity(128),
        };

        Ok(Self(RwLock::new(inner)))
    }

    pub async fn print(&self, text: &str) -> Result<()> {
        let mut inner = self.0.write().await;
        let _lock = graphics::get_instance().lock().await;

        Self::print_internal(&mut inner, text).await?;

        Ok(())
    }

    fn get_start_index(buffer: &mut String, text: &str) {
        let start_index = if let Some(last_clear) = text.rfind(Self::CLEAR) {
            buffer.clear();
            last_clear + Self::CLEAR.len()
        } else {
            0
        };

        let start_index = if let Some(last_home) = text.rfind(Self::HOME) {
            start_index.max(last_home + Self::HOME.len())
        } else {
            start_index
        };

        let text = &text[start_index..];

        buffer.push_str(text);
    }

    async fn print_internal(inner: &mut Inner, text: &str) -> Result<()> {
        inner.buffer.pop(); // Remove the trailing null character

        Self::get_start_index(&mut inner.buffer, text);
        inner.buffer.push('\0');

        unsafe {
            lvgl::lv_label_set_text_static(inner.display, inner.buffer.as_ptr() as *const i8);
            lvgl::lv_obj_scroll_to_view(inner.display, true);
        }

        Ok(())
    }

    async fn print_line_internal(inner: &mut Inner, text: &str) -> Result<()> {
        inner.buffer.pop(); // Remove the trailing null character

        Self::get_start_index(&mut inner.buffer, text);
        inner.buffer.push_str("\n\0");

        unsafe {
            lvgl::lv_label_set_text_static(inner.display, inner.buffer.as_ptr() as *const i8);
            lvgl::lv_obj_scroll_to_view(inner.display, true);
        }

        Ok(())
    }

    pub async fn read_input(&self, buffer: &mut [u8]) -> Result<Size> {
        let mut inner = self.0.write().await;

        let mut read = 0;

        buffer
            .iter_mut()
            .zip(inner.validated_input.iter())
            .for_each(|(byte, &char)| {
                *byte = char;
                read += 1;
            });

        inner.validated_input.drain(0..read);

        Ok(Size::new(read as u64))
    }

    fn get_input(text_area: *mut lv_obj_t) -> &'static str {
        unsafe {
            let text = lvgl::lv_textarea_get_text(text_area);

            CStr::from_ptr(text).to_str().unwrap_or("")
        }
    }

    pub async fn event_handler(&self) -> Result<bool> {
        let mut inner = self.0.write().await;

        while let Some(event) = inner.window.pop_event() {
            match event.get_code() {
                EventKind::Delete => return Ok(false),
                EventKind::Key => {
                    if let Some(Key::Enter) = event.get_key() {
                        let _lock = graphics::get_instance().lock().await;

                        let text = Self::get_input(inner.input);

                        Self::print_line_internal(&mut inner, text).await?;

                        let text = Self::get_input(inner.input);

                        inner.validated_input.extend(text.as_bytes());
                        inner.validated_input.push_back(b'\n');

                        unsafe {
                            lvgl::lv_textarea_set_text(inner.input, c"".as_ptr());
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }
}
