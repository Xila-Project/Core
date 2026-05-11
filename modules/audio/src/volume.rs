pub use crate::Sample;

pub unsafe fn apply_volume(sample: [Sample; 8], volume: f32) -> [Sample; 8] {
    let volume = volume.clamp(0.0, 1.0);
    let left = (sample[0] as f32 * volume).round() as Sample;
    let right = (sample[1] as f32 * volume).round() as Sample;
    [left, right]
}
