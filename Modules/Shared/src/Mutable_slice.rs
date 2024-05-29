#![allow(non_camel_case_types)]

use std::{
    fmt::{Debug, Display, Formatter},
    ptr::NonNull,
};

#[derive(Debug)]
pub struct Mutable_slice_type<'a, T> {
    Data: &'a mut [T],
    Length: &'a mut u32,
}

impl<'a, T> Mutable_slice_type<'a, T> {
    pub fn From(Slice: NonNull<T>, mut Length: NonNull<u32>, Size: u32) -> Result<Self, String> {
        let Data: &'a mut [T] =
            unsafe { std::slice::from_raw_parts_mut(Slice.as_ptr(), Size as usize) };

        if unsafe { *Length.as_ref() } > Data.len() as u32 {
            return Err("Invalid length".to_string());
        }

        Ok(Self {
            Data,
            Length: unsafe { Length.as_mut() },
        })
    }

    pub fn Push(&mut self, Value: T) -> Result<(), ()> {
        if self.Get_length() == self.Get_size() {
            return Err(());
        }

        self.Data[self.Get_length()] = Value;
        self.Set_length(self.Get_length() + 1);

        Ok(())
    }

    fn Set_length(&mut self, Length: usize) {
        *self.Length = Length as u32;
    }

    pub fn Clear(&mut self) {
        self.Set_length(0)
    }

    pub fn Is_empty(&self) -> bool {
        self.Get_length() == 0
    }

    pub fn Get_length(&self) -> usize {
        *self.Length as usize
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
    fn test_mutable_slice() {
        let mut Data = [1, 2, 3, 4, 5];
        let mut Length = 3;

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
