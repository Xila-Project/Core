use core::slice;

use alloc::{string::String, sync::Arc, vec::Vec};
use file_system::DeviceTrait;
use futures::block_on;
use graphics::{Area, Point, RenderingColor, ScreenReadData, ScreenWriteData};
use synchronization::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

struct Inner {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    conversion_buffer: Vec<u8>,
}

impl Inner {
    fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &JsValue::FALSE).unwrap();

        let context = canvas
            .get_context_with_context_options("2d", &options)
            .unwrap()
            .ok_or("No 2D context found")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;

        Ok(Self {
            canvas,
            context,
            conversion_buffer: Vec::new(),
        })
    }

    fn get_resolution(&self) -> Result<Point, String> {
        Ok(Point::new(
            self.canvas.width() as _,
            self.canvas.height() as _,
        ))
    }

    #[target_feature(enable = "simd128")]
    unsafe fn rgba_to_bgra_simd(rgba: &[u8], bgra: &mut [u8]) {
        use core::arch::wasm32::*;

        let len = rgba.len();
        let mut i = 0;
        while i + 16 <= len {
            let v = unsafe { v128_load(rgba.as_ptr().add(i) as *const v128) };
            // shuffle mask: swap R and B (0 <-> 2)
            let swapped =
                i8x16_shuffle::<2, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15>(v, v);
            unsafe {
                v128_store(bgra.as_mut_ptr().add(i) as *mut v128, swapped);
            }
            i += 16;
        }

        // handle remainder
        for j in i..len {
            bgra[j] = rgba[j];
        }
    }

    fn draw_buffer(&mut self, area: Area, buffer: &[RenderingColor]) -> Result<(), String> {
        let pixel_count = area.get_width() as usize * area.get_height() as usize;
        let bytes_needed = pixel_count * size_of::<RenderingColor>();

        if self.conversion_buffer.capacity() < bytes_needed {
            self.conversion_buffer
                .reserve(bytes_needed - self.conversion_buffer.len());
        }
        unsafe {
            self.conversion_buffer.set_len(bytes_needed);
        }

        let destination = unsafe {
            slice::from_raw_parts_mut(self.conversion_buffer.as_mut_ptr() as *mut u8, bytes_needed)
        };

        let source = unsafe { slice::from_raw_parts(buffer.as_ptr() as *const u8, bytes_needed) };

        unsafe {
            Self::rgba_to_bgra_simd(source, destination);
        }

        // now conversion_buffer is ready as &[u8]
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.conversion_buffer),
            area.get_width() as _,
            area.get_height() as _,
        )
        .expect("Failed to create ImageData");

        let (x, y) = area.get_point_1().into();
        self.context
            .put_image_data(&image_data, x as _, y as _)
            .unwrap();

        Ok(())
    }
}

pub struct CanvasScreenDevice(Arc<RwLock<CriticalSectionRawMutex, Inner>>);

unsafe impl Sync for CanvasScreenDevice {}
unsafe impl Send for CanvasScreenDevice {}

impl CanvasScreenDevice {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        Ok(Self(Arc::new(RwLock::new(Inner::new(canvas)?))))
    }
}

impl DeviceTrait for CanvasScreenDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        let data: &mut ScreenReadData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        let inner = block_on(self.0.read());

        let resolution = inner
            .get_resolution()
            .map_err(|_| file_system::Error::InputOutput)?;

        data.set_resolution(resolution);

        Ok(file_system::Size::new(buffer.len() as u64))
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        let screen_data: &ScreenWriteData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        let mut inner = block_on(self.0.write());

        inner
            .draw_buffer(screen_data.get_area(), screen_data.get_buffer())
            .map_err(|_| file_system::Error::InputOutput)?;

        Ok(file_system::Size::new(buffer.len() as u64))
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        let inner = block_on(self.0.read());
        let (width, height) = inner
            .get_resolution()
            .map_err(|_| file_system::Error::InputOutput)?
            .into();
        Ok(file_system::Size::new(
            (width as usize * height as usize * size_of::<RenderingColor>()) as u64,
        ))
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<file_system::Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
