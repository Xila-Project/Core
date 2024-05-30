#![allow(non_camel_case_types)]

use std::{
    fmt::{Debug, Display, Formatter},
    ptr::NonNull,
};

use num::{Num, NumCast, ToPrimitive, Unsigned};

use crate::Error_type;

#[derive(Debug)]
pub struct Mutable_slice_type<'a, T, S = u32>
where
    S: Unsigned + Num + NumCast + PartialOrd + ToPrimitive + Copy,
{
    Data: &'a mut [T],
    Length: &'a mut S,
}

impl<'a, T, S> Mutable_slice_type<'a, T, S>
where
    S: Unsigned + Num + NumCast + Num + PartialOrd + ToPrimitive + Copy,
{
    pub fn From(Slice: NonNull<T>, mut Length: NonNull<S>, Size: S) -> Result<Self, Error_type> {
        let Data: &'a mut [T] = unsafe {
            std::slice::from_raw_parts_mut(
                Slice.as_ptr(),
                Size.to_usize()
                    .ok_or(Error_type::Failed_to_convert_length_to_S)?,
            )
        };

        if unsafe { *Length.as_ref() } > Size {
            return Err(Error_type::Invalid_length);
        }

        Ok(Self {
            Data,
            Length: unsafe { Length.as_mut() },
        })
    }

    pub fn Push(&mut self, Value: T) -> Result<(), Error_type> {
        if self.Get_length() >= self.Get_size() {
            return Err(Error_type::Buffer_too_small);
        }

        self.Data[self.Get_length()] = Value;
        self.Set_length(
            S::from(self.Get_length()).ok_or(Error_type::Failed_to_convert_length_to_S)?
                + S::from(1).unwrap(),
        )
    }

    fn Set_length(&mut self, Length: S) -> Result<(), Error_type> {
        if Length.to_usize().unwrap() > self.Get_size() {
            return Err(Error_type::Invalid_length);
        }

        *self.Length = Length as S;
        Ok(())
    }

    pub fn Clear(&mut self) {
        self.Set_length(S::from(0).unwrap()).unwrap();
    }

    pub fn Is_empty(&self) -> bool {
        self.Get_length() == 0
    }

    pub fn Get_length(&self) -> usize {
        (*self.Length).to_usize().unwrap()
    }

    pub fn Get_size(&self) -> usize {
        self.Data.len()
    }

    pub fn As_slice(&self) -> &[T] {
        &self.Data[..self.Get_length()]
    }
}

impl<T: Debug> Display for Mutable_slice_type<'_, T> {
    fn fmt(&self, Formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(Formatter, "{:?}", self.As_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_mutable_slice() {
        let mut Data = [1, 2, 3, 4, 5];
        let mut Length = 3 as u32;

        let Raw_slice = NonNull::new(Data.as_mut_ptr()).unwrap();
        let Length = NonNull::new(&mut Length).unwrap();

        let mut Slice = Mutable_slice_type::From(Raw_slice, Length, 5).unwrap();

        assert_eq!(Slice.Get_length(), 3);
        assert_eq!(Slice.Get_size(), 5);
        assert_eq!(Slice.As_slice(), &[1, 2, 3]);

        Slice.Clear();

        assert_eq!(Slice.Get_length(), 0);
        assert_eq!(Slice.Is_empty(), true);

        Slice.Push(1).unwrap();

        assert_eq!(Slice.Get_length(), 1);
        assert_eq!(Slice.As_slice(), &[1]);

        Slice.Push(2).unwrap();

        assert_eq!(Slice.Get_length(), 2);
        assert_eq!(Slice.As_slice(), &[1, 2]);
    }
}
