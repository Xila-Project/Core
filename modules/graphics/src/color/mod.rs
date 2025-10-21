mod argb8888;
pub mod palette;
mod rgb565;
mod rgb888;
mod rgba8888;

pub use argb8888::*;
pub use rgb565::*;
pub use rgb888::*;
pub use rgba8888::*;

pub type Color = ColorRGB888;

#[cfg(feature = "rendering_rgb565")]
pub type RenderingColor = ColorRGB565;
#[cfg(feature = "rendering_xrgb8888")]
pub type RenderingColor = ColorARGB8888;
