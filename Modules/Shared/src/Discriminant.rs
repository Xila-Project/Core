use std::num::NonZeroU32;

pub trait Error_discriminant_trait: Clone {
    fn Get_discriminant(&self) -> NonZeroU32;
    fn From_discriminant(Discriminant: NonZeroU32) -> Self;
}

pub trait Discriminant_trait {
    fn Get_discriminant(&self) -> u32;
    fn From_discriminant(Discriminant: u32) -> Self;
}

pub fn From_result_to_u32<T, E: Error_discriminant_trait>(Result: &Result<T, E>) -> u32 {
    match Result {
        Ok(_) => 0,
        Err(Error) => Error.Get_discriminant().get(),
    }
}
