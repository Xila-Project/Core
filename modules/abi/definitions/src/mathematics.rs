#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_absolute_value(value: i32) -> i32 {
    value.abs()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn xila_mathematics_is_not_a_number(value: f32) -> bool {
    value.is_nan()
}
