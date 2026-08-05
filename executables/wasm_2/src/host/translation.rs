use wasmi::{AsContextMut, Caller};

use crate::host::error::{Error, Result};

pub type WasmUsize = u32;
pub type WasmPointer = u32;

pub fn get_memory<'a, T>(
    caller: &'a mut Caller<T>,
) -> core::result::Result<&'a mut [u8], wasmi::Error> {
    Ok(caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| wasmi::Error::new("missing memory"))?
        .data_mut(caller.as_context_mut()))
}

pub trait TranslateFrom<'a, I> {
    unsafe fn translate_from(parameters: I, memory: &'a mut [u8]) -> Result<Self>
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
            impl<'a> TranslateFrom<'a, WasmUsize> for $t{
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
            impl<'a> TranslateFrom<'a, WasmUsize> for *mut $t {
                #[inline]
                unsafe fn translate_from(pointer: WasmUsize, memory: &'a mut [u8]) -> Result<Self> {
                    if pointer == 0 {
                        return Ok(core::ptr::null_mut());
                    }

                    let start = pointer as usize;
                    let end = start
                        .checked_add(core::mem::size_of::<$t>())
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    let slice = memory.get_mut(start..end)
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    if slice.as_mut_ptr() as usize % core::mem::align_of::<$t>() != 0 {
                        return Err(Error::UnalignedTranslation);
                    }

                    Ok(slice.as_mut_ptr() as *mut $t)
                }
            }

            impl<'a> TranslateFrom<'a, WasmUsize> for *const $t {
                #[inline]
                unsafe fn translate_from(pointer: WasmUsize, memory: &'a mut [u8]) -> Result<Self> {
                    if pointer == 0 {
                        return Ok(core::ptr::null());
                    }

                    let start = pointer as usize;
                    // Was a plain `+`, which could overflow before the bounds check
                    // ever runs. `*mut` sibling already used checked_add; mirrored here.
                    let end = start
                        .checked_add(core::mem::size_of::<$t>())
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    // `.get()` instead of `.get_mut()` — we only need a `*const`, no
                    // need to require exclusive access to produce it.
                    let slice = memory.get(start..end)
                        .ok_or(Error::OutOfBoundsTranslation)?;

                    if slice.as_ptr() as usize % core::mem::align_of::<$t>() != 0 {
                        return Err(Error::UnalignedTranslation);
                    }

                    Ok(slice.as_ptr() as *const $t)
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

pub struct TranslationSliceIterator<'a, S, D> {
    memory: &'a mut [u8],
    _marker: core::marker::PhantomData<(S, D)>,
}

impl<'a, S, D> TranslateFrom<'a, (WasmUsize, WasmUsize)> for TranslationSliceIterator<'a, S, D>
where
    D: Sized + TranslateFrom<'a, S>,
{
    #[inline]
    unsafe fn translate_from(
        (pointer, length): (WasmUsize, WasmUsize),
        data: &'a mut [u8],
    ) -> Result<Self> {
        let start = pointer as usize;

        // `length` is guest-controlled: multiply with `checked_mul` before it
        // ever reaches `checked_add`, or it can overflow/wrap first.
        let byte_len = (length as usize)
            .checked_mul(core::mem::size_of::<S>())
            .ok_or(Error::OutOfBoundsTranslation)?;

        let end = start
            .checked_add(byte_len)
            .ok_or(Error::OutOfBoundsTranslation)?;

        // Bounds-check as an offset into `data` (consistent with every other
        // impl in this file), not against `data.as_ptr()`'s absolute host
        // address — comparing offsets to an absolute pointer value made this
        // reject essentially all legitimate input.
        let slice = data
            .get_mut(start..end)
            .ok_or(Error::OutOfBoundsTranslation)?;

        // Alignment check was missing entirely for this path; every other
        // translation impl in the file enforces one.
        if slice.as_mut_ptr() as usize % core::mem::align_of::<S>() != 0 {
            return Err(Error::UnalignedTranslation);
        }

        Ok(Self {
            memory: slice,
            _marker: core::marker::PhantomData,
        })
    }
}

impl<'a, S, D> Iterator for TranslationSliceIterator<'a, S, D>
where
    D: Sized + TranslateFrom<'a, S>,
{
    type Item = Result<D>;

    fn next(&mut self) -> Option<Self::Item> {
        let elem_size = core::mem::size_of::<S>();
        if self.memory.len() < elem_size {
            return None;
        }

        // `&mut self.memory[..]` would be a *reborrow* through `&mut self`,
        // whose lifetime is bounded by this call to `next`, not by `'a` —
        // that's the "lifetime may not live long enough" error, since
        // `D: TranslateFrom<'a, S>` needs a genuine `&'a mut [u8]`.
        //
        // `mem::take` moves the real `&'a mut [u8]` out of `self` (leaving
        // an empty slice behind), so splitting it keeps the original `'a`
        // lifetime on both halves. We hand out `current` and store `rest`
        // back for the next call.
        let memory = core::mem::take(&mut self.memory);
        let (current, rest) = memory.split_at_mut(elem_size);
        self.memory = rest;

        let parameters = unsafe { core::ptr::read_unaligned(current.as_ptr() as *const S) };

        Some(unsafe { D::translate_from(parameters, current) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.memory.len() / core::mem::size_of::<S>();
        (remaining, Some(remaining))
    }
}

impl<'a, S, D> ExactSizeIterator for TranslationSliceIterator<'a, S, D> where
    D: Sized + TranslateFrom<'a, S>
{
}
