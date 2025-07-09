pub type Result_type<T> = core::result::Result<T, Error_type>;

#[derive(Debug, Clone, Copy)]
pub enum Error_type {
    Failed_to_register_pin_device,
}
