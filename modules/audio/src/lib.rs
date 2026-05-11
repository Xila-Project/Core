mod mixing;
mod resampling;
mod volume;

#[cfg(feature = "i16-samples")]
pub type Sample = i16;

#[cfg(feature = "i32-samples")]
pub type Sample = i32;
