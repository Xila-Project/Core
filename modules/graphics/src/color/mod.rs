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

#[cfg(not(any(feature = "rendering_rgb565", feature = "rendering_xrgb8888")))]
compile_error!("Either feature \"rendering_rgb565\" or \"rendering_xrgb8888\" must be enabled");
