#![allow(non_camel_case_types)]

use core::slice;
use std::{
    fmt::Display,
    ops::AddAssign,
    ptr::NonNull,
    str::{Chars, Lines},
};

/// This structure is a safe wrapper for a mutable string slice from external pointers.
///
/// This is specifically designed to be used with foreign function interfaces.
#[derive(Debug, Eq)]
pub struct Mutable_string_type<'a> {
    Data: &'a mut [u8],
    // This is the actual length of the string, not the size of the buffer.
    Length: &'a mut u32,
}

impl<'a> Mutable_string_type<'a> {
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the buffer contains a valid UTF-8 string and if the length is valid.
    pub unsafe fn From_unchecked(String: NonNull<u8>, mut Length: NonNull<u32>, Size: u32) -> Self {
        let Data: &'a mut [u8] =
            unsafe { slice::from_raw_parts_mut(String.as_ptr(), Size as usize) };

        Self {
            Data,
            Length: Length.as_mut(),
        }
    }

    pub fn From(String: NonNull<u8>, mut Length: NonNull<u32>, Size: u32) -> Result<Self, String> {
        let Data: &'a mut [u8] =
            unsafe { slice::from_raw_parts_mut(String.as_ptr(), Size as usize) };

        if std::str::from_utf8(Data).is_err() {
            return Err("Invalid UTF-8 string".to_string());
        }

        if unsafe { *Length.as_ref() } > Data.len() as u32 {
            return Err("Invalid length".to_string());
        }

        Ok(Self {
            Data,
            Length: unsafe { Length.as_mut() },
        })
    }

    pub fn Equal(&self, String: &str) -> bool {
        self.As_str() == String
    }

    pub fn Concatenate(&mut self, String: &str) -> Result<(), String> {
        let String = String.as_bytes();

        let Length = String.len();

        if self.Get_length() + Length > self.Get_size() {
            return Err("Buffer too small".to_string());
        }

        let Self_length = self.Get_length();

        self.Data[Self_length..Self_length + Length].copy_from_slice(String);

        self.Set_length(
            u32::try_from(self.Get_length() + Length)
                .map_err(|_| "Failed to convert length to u32".to_string())?,
        );

        Ok(())
    }

    pub fn Clear(&mut self) {
        self.Set_length(0)
    }

    fn Set_length(&mut self, Length: u32) {
        *self.Length = Length as u32;
    }

    pub fn Get_data(&'a mut self) -> &'a mut [u8] {
        self.Data
    }

    pub fn Get_length(&self) -> usize {
        *self.Length as usize
    }

    pub fn Get_size(&self) -> usize {
        self.Data.len() as usize
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
impl AddAssign<&str> for Mutable_string_type<'_> {
    fn add_assign(&mut self, Other: &str) {
        let _ = self.Concatenate(Other);
    }
}

impl Display for Mutable_string_type<'_> {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(Formatter, "{}", self.As_str())
    }
}

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
