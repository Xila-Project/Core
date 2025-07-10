pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Failed_to_register_pin_device,
}
