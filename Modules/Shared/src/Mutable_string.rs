use core::slice;
use std::{
    fmt::Display,
    ops::AddAssign,
    ptr::NonNull,
    str::{Chars, Lines},
};

use crate::{Error_type, Size_trait};

/// This structure is a safe wrapper for a mutable string slice from external pointers.
///
/// This is specifically designed to be used with foreign function interfaces.
#[derive(Debug)]
pub struct Mutable_string_type<'a, S = u32>
where
    S: Size_trait,
{
    Data: &'a mut [u8],
    // This is the actual length of the string, not the size of the buffer.
    Length: &'a mut S,
}

impl<'a, S> Mutable_string_type<'a, S>
where
    S: Size_trait,
{
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the buffer contains a valid UTF-8 string and if the length is valid.
    pub unsafe fn From_unchecked(String: NonNull<u8>, mut Length: NonNull<S>, Size: S) -> Self {
        let Data: &'a mut [u8] =
            unsafe { slice::from_raw_parts_mut(String.as_ptr(), Size.to_usize().unwrap()) };

        Self {
            Data,
            Length: Length.as_mut(),
        }
    }

    pub fn From(String: NonNull<u8>, mut Length: NonNull<S>, Size: S) -> Result<Self, Error_type> {
        let Size = Size
            .to_usize()
            .ok_or(Error_type::Failed_to_convert_length_to_S)?;

        let Casted_length = unsafe { (*Length.as_ref()).to_usize().unwrap() };

        let Data: &'a mut [u8] = unsafe { slice::from_raw_parts_mut(String.as_ptr(), Size) };

        if std::str::from_utf8(&Data[..Casted_length]).is_err() {
            return Err(Error_type::Invalid_UTF8_string);
        }

        if Casted_length > Size {
            return Err(Error_type::Invalid_length);
        }

        Ok(Self {
            Data,
            Length: unsafe { Length.as_mut() },
        })
    }

    pub fn Equal(&self, String: &str) -> bool {
        self.As_str() == String
    }

    pub fn Concatenate(&mut self, String: &str) -> Result<(), Error_type> {
        let String = String.as_bytes();

        let Length = String.len();

        if self.Get_length() + Length > self.Get_size() {
            return Err(Error_type::Buffer_too_small);
        }

        let Self_length = self.Get_length();

        self.Data[Self_length..Self_length + Length].copy_from_slice(String);

        self.Set_length(
            S::from(self.Get_length() + Length).ok_or(Error_type::Failed_to_convert_length_to_S)?,
        );

        Ok(())
    }

    pub fn Clear(&mut self) {
        self.Set_length(S::from(0).unwrap());
    }

    fn Set_length(&mut self, Length: S) {
        *self.Length = Length;
    }

    pub fn Get_data(&'a mut self) -> &'a mut [u8] {
        self.Data
    }

    pub fn Get_length(&self) -> usize {
        (*self.Length).to_usize().unwrap()
    }

    pub fn Get_size(&self) -> usize {
        self.Data.len()
    }

    pub fn Get_characters(&self) -> Chars {
        self.As_str().chars()
    }

    pub fn Get_characters_indices(&self) -> impl Iterator<Item = (usize, char)> + '_ {
        self.As_str().char_indices()
    }

    pub fn Get_lines(&'a self) -> Lines<'a> {
        self.As_str().lines()
    }

    pub fn As_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(
                self.Data.as_ptr(),
                self.Get_length(),
            ))
        }
    }
}

/// Implement the `Add` operator for `Mutable_string_slice_type`.
///
/// # Safety
///
/// This function can fail silently if the buffer is not large enough.
impl<S> AddAssign<&str> for Mutable_string_type<'_, S>
where
    S: Size_trait,
{
    fn add_assign(&mut self, Other: &str) {
        let _ = self.Concatenate(Other);
    }
}

impl Display for Mutable_string_type<'_> {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(Formatter, "{}", self.As_str())
    }
}

impl Eq for Mutable_string_type<'_> {}

impl PartialEq for Mutable_string_type<'_> {
    fn eq(&self, Other: &Self) -> bool {
        self.As_str() == Other.As_str()
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_upper_case_globals)]

    const Test_str_1: &str = "Hello, World!";
    const Test_str_2: &str = "This is a new line.";
    const Test_str_3: &str = "Some UTF-8 characters: 🦀💓";
    const Test_str_4: &str = "More UTF-8 characters: 🦀💓";

    use super::*;

    #[test]
    fn Test_all() {
        let Test_string_4 = Test_str_4.to_string();

        let mut Buffer = [0u8; 100];

        let mut len: u32 = 0;

        let mut String = unsafe {
            Mutable_string_type::From_unchecked(
                std::ptr::NonNull::new(Buffer.as_mut_ptr()).unwrap(),
                std::ptr::NonNull::new(&mut len as *mut u32).unwrap(),
                100,
            )
        };

        assert_eq!(String.Get_size(), 100);

        assert_eq!(String.Get_length(), 0);

        assert_eq!(String.Get_characters().count(), 0);

        assert_eq!(String.As_str(), "");

        String += Test_str_1;

        assert_eq!(String.Get_length(), Test_str_1.len());

        assert_eq!(String.Get_characters().count(), Test_str_1.chars().count());

        assert_eq!(String.As_str(), Test_str_1);

        String += "\n";
        String += "This is a new line.";
        String += "\n";
        String += &Test_str_3;
        String += "\n";
        String += &Test_string_4;

        for (i, Line) in String.Get_lines().enumerate() {
            match i {
                0 => assert_eq!(Line, Test_str_1),
                1 => assert_eq!(Line, Test_str_2),
                2 => assert_eq!(Line, Test_str_3),
                3 => assert_eq!(Line, &Test_string_4),
                _ => panic!("Unexpected line: {}", Line),
            }
        }

        assert_eq!(
            String.Get_length(),
            Test_str_1.len() + Test_str_2.len() + Test_str_3.len() + Test_str_4.len() + 3
        );

        assert_eq!(
            String.Get_characters().count(),
            Test_str_1.chars().count()
                + Test_str_2.chars().count()
                + Test_str_3.chars().count()
                + Test_str_4.chars().count()
                + 3
        );

        String.Clear();

        assert_eq!(String.Get_length(), 0);

        assert_eq!(String.Get_characters().count(), 0);

        assert_eq!(String.As_str(), "");
    }
}
