use wasmi::Caller;

use crate::host::error::{Error, Result};

pub type WasmUsize = u32;
pub type WasmPointer = u32;

pub fn get_memory<T>(caller: &Caller<T>) -> core::result::Result<wasmi::Memory, wasmi::Error> {
    caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| wasmi::Error::new("missing memory"))
}

pub trait TranslateFrom<I> {
    unsafe fn translate_from(parameters: I, memory: &mut [u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait TranslateInto<I>: Sized {
    type Output;

    unsafe fn translate_into(self, memory: &mut [u8]) -> Result<Self::Output>;
}

macro_rules! implicit_usize_cast {
    ($($t:ty),* $(,)?) => {
        $(
            impl TranslateFrom<WasmUsize> for $t{
                #[inline]
                unsafe fn translate_from(parameters: WasmUsize, _: &mut [u8]) -> Result<Self> {
                    Ok(parameters as $t)
                }
            }

            impl TranslateInto<WasmUsize> for $t {
                type Output = Self;
                #[inline]
                unsafe fn translate_into(self, _: &mut [u8]) -> Result<Self::Output> {
                    Ok(self as $t)
                }
            }
        )*
    };
}

macro_rules! implicit_pointer_translation {
    ($($t:ty),* $(,)?) => {
        $(
            impl TranslateFrom<WasmUsize> for *mut $t {
                #[inline]
                unsafe fn translate_from(pointer: WasmUsize, memory: &mut [u8]) -> Result<Self> {
                    if pointer == 0 {
                        return Ok(core::ptr::null_mut());
                    }

                    let stard = pointer as usize;
                    let end = (pointer as usize).checked_add(core::mem::size_of::<$t>())
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    let slice = memory.get_mut(pointer as usize..end)
                        .ok_or(Error::OutOfBoundsTranslation)?;


                    if slice.as_mut_ptr() as usize % core::mem::align_of::<$t>() != 0 {
                        return Err(Error::UnalignedTranslation);
                    }

                    Ok(slice.as_mut_ptr() as *mut $t)
                }
            }

            impl TranslateFrom<WasmUsize> for *const $t {
                #[inline]
                unsafe fn translate_from(wasm_usize: WasmUsize, memory: &mut [u8]) -> Result<Self> {
                    if wasm_usize == 0 {
                        return Ok(core::ptr::null());
                    }

                    let slice = memory.get_mut(wasm_usize as usize..wasm_usize as usize + core::mem::size_of::<$t>())
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    if slice.as_mut_ptr() as usize % core::mem::align_of::<$t>() != 0 {
                        return Err(Error::UnalignedTranslation);
                    }

                    Ok(slice.as_mut_ptr() as *const $t)
                }
            }

            impl TranslateInto<WasmUsize> for *mut $t {
                type Output = WasmUsize;

                #[inline]
                unsafe fn translate_into(self, memory: &mut [u8]) -> Result<Self::Output> {
                    if self.is_null() {
                        return Ok(0);
                    }

                    let offset = (self as usize).checked_sub(memory.as_ptr() as usize)
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    if offset + core::mem::size_of::<$t>() > memory.len() {
                        return Err(Error::OutOfBoundsTranslation);
                    }

                    if (self as usize) % core::mem::align_of::<$t>() != 0 {
                        return Err(Error::UnalignedTranslation);
                    }

                    Ok(offset as WasmUsize)
                }
            }

            impl TranslateInto<WasmUsize> for *const $t {
                type Output = WasmUsize;

                #[inline]
                unsafe fn translate_into(self, memory: &mut [u8]) -> Result<Self::Output> {
                    if self.is_null() {
                        return Ok(0);
                    }

                    let offset = (self as usize).checked_sub(memory.as_ptr() as usize)
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    if offset + core::mem::size_of::<$t>() > memory.len() {
                        return Err(Error::OutOfBoundsTranslation);
                    }

                    if (self as usize) % core::mem::align_of::<$t>() != 0 {
                        return Err(Error::UnalignedTranslation);
                    }

                    Ok(offset as WasmUsize)
                }
            }
        )*
    };
}

implicit_pointer_translation!(
    core::ffi::c_void,
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
);

implicit_usize_cast!(u8, u16, u32, usize, i8, i16, i32, isize, f32);

#[cfg(target_pointer_width = "64")]
implicit_usize_cast!(u64, i64, f64);

impl<T> TranslateFrom<(WasmUsize, WasmUsize)> for &mut [T] {
    #[inline]
    unsafe fn translate_from(
        (pointer, length): (WasmUsize, WasmUsize),
        data: &mut [u8],
    ) -> Result<Self> {
        let start = pointer as usize;
        let end = start
            .checked_add(length as usize * core::mem::size_of::<T>())
            .ok_or(Error::OutOfBoundsTranslation)?;

        let slice = data
            .get_mut(start..end)
            .ok_or(Error::OutOfBoundsTranslation)?;

        if slice.as_mut_ptr() as usize % core::mem::align_of::<T>() != 0 {
            return Err(Error::UnalignedTranslation);
        }

        Ok(unsafe {
            core::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut T, length as usize)
        })
    }
}

impl<T> TranslateInto<(WasmUsize, WasmUsize)> for &mut [T] {
    type Output = (WasmUsize, WasmUsize);

    #[inline]
    unsafe fn translate_into(self, data: &mut [u8]) -> Result<Self::Output> {
        let start = self.as_mut_ptr() as usize;
        let end = start
            .checked_add(self.len() * core::mem::size_of::<T>())
            .ok_or(Error::OutOfBoundsTranslation)?;

        if start < data.as_ptr() as usize || end > data.as_ptr() as usize + data.len() {
            return Err(Error::OutOfBoundsTranslation);
        }

        if start % core::mem::align_of::<T>() != 0 {
            return Err(Error::UnalignedTranslation);
        }

        let start = start
            .checked_sub(data.as_ptr() as usize)
            .ok_or(Error::OutOfBoundsTranslation)?;

        Ok((start as WasmUsize, self.len() as WasmUsize))
    }
}
