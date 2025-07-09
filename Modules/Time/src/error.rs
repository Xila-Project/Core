#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error_type {
    Not_initialized,
    Already_initialized,
    Device_error(file_system::Error_type),
}

pub type Result_type<T> = Result<T, Error_type>;
