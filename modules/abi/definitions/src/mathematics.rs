#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_absolute_value(value: i32) -> i32 {
    value.abs()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_is_not_a_number(value: f32) -> bool {
    value.is_nan()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_absolute_value_f32(value: f32) -> f32 {
    value.abs()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_ceiling_f32(value: f32) -> f32 {
    value.ceil()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_floor_f32(value: f32) -> f32 {
    value.floor()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_truncate_f32(value: f32) -> f32 {
    value.trunc()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_round_f32(value: f32) -> f32 {
    value.round()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_square_root_f32(value: f32) -> f32 {
    value.sqrt()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_absolute_value_f64(value: f64) -> f64 {
    value.abs()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_ceiling_f64(value: f64) -> f64 {
    value.ceil()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_floor_f64(value: f64) -> f64 {
    value.floor()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_truncate_f64(value: f64) -> f64 {
    value.trunc()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_round_f64(value: f64) -> f64 {
    value.round()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_square_root_f64(value: f64) -> f64 {
    value.sqrt()
}
