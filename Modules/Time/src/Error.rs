#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error_type {
    Not_initialized,
    Already_initialized,
}

pub type Result_type<T> = Result<T, Error_type>;
