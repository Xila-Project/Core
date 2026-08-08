use core::fmt::Display;

use wasmi::{AsContextMut, Caller, errors::HostError};

pub type WasmUsize = u32;
pub type WasmPointer = u32;

#[derive(Debug, Clone, Copy)]
pub enum TranslationError {
    Unaligned,
    OutOfBounds,
    Other,
}

impl core::error::Error for TranslationError {}

impl Display for TranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unaligned => f.write_str("unaligned translation"),
            Self::OutOfBounds => f.write_str("out-of-bounds translation"),
            Self::Other => f.write_str("other translation error"),
        }
    }
}

impl HostError for TranslationError {}

impl From<TranslationError> for wasmi::Error {
    fn from(err: TranslationError) -> Self {
        wasmi::Error::host(err)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WasiVector {
    pub buffer: WasmPointer,
    pub length: WasmUsize,
}

impl WasiVector {
    pub fn new(buffer: WasmPointer, length: WasmUsize) -> Self {
        Self { buffer, length }
    }
}

// Both fields are plain u32s with no niches, so every bit pattern is a
// valid `WasiVector` — needed because `TranslationSliceStream` below reads
// `WasiVector` values directly from guest bytes via `read_unaligned`.
impl sealed::Sealed for WasiVector {}
impl WasmPod for WasiVector {}

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
    /// # Safety
    /// `memory` must be the full linear memory currently backing the wasm
    /// instance for this call. The returned value (in particular, any raw
    /// pointer) must not be used after this host call returns, or after any
    /// wasm code runs that could grow/reallocate this memory.
    unsafe fn translate_from(parameters: I, memory: &'a mut [u8]) -> Result<Self, TranslationError>
    where
        Self: Sized;
}

pub trait TranslateInto<I>: Sized {
    type Output;

    /// # Safety
    /// If `self` is a pointer, it must have been produced by a matching
    /// `TranslateFrom` call against this same `memory`, and `memory` must
    /// not have grown/moved since.
    unsafe fn translate_into(self, memory: &mut [u8]) -> Result<Self::Output, TranslationError>;
}

mod sealed {
    pub trait Sealed {}
}

/// Types for which *every* bit pattern is a valid value — i.e. safe to
/// materialize as a value or a reference from arbitrary, guest-controlled
/// bytes. Sealed: only this module may vouch for a type, so extending the
/// set requires touching (and reviewing) this file. This is what stops a
/// type like `core::ffi::c_void` — which has only two valid bit patterns —
/// from ending up somewhere that reads or references it directly.
pub trait WasmPod: sealed::Sealed + Copy {}

/// Bounds-check + align-check `start..start + count * size_of::<T>()`
/// against `memory`, returning the validated sub-slice. Every translation
/// below (single pointer, slice, and the array iterator) goes through this
/// one function, so they can't drift apart the way the mut/const pointer
/// paths previously did.
#[inline]
fn validate_and_slice_n<T>(
    start: usize,
    count: usize,
    memory: &mut [u8],
) -> Result<&mut [u8], TranslationError> {
    let byte_len = count
        .checked_mul(core::mem::size_of::<T>())
        .ok_or(TranslationError::OutOfBounds)?;

    let end = start
        .checked_add(byte_len)
        .ok_or(TranslationError::OutOfBounds)?;

    let slice = memory
        .get_mut(start..end)
        .ok_or(TranslationError::OutOfBounds)?;

    if slice.as_mut_ptr() as usize % core::mem::align_of::<T>() != 0 {
        return Err(TranslationError::Unaligned);
    }

    Ok(slice)
}

#[inline]
fn validate_and_slice<T>(start: usize, memory: &mut [u8]) -> Result<&mut [u8], TranslationError> {
    validate_and_slice_n::<T>(start, 1, memory)
}

/// Computes the wasm-side offset of a host pointer, checked for overflow
/// and bounds. This can only check that the arithmetic lands within
/// `memory` — it cannot *prove* `pointer` actually originated from
/// `memory`. Only pass in pointers this module produced via a matching
/// `TranslateFrom` call against the same memory.
#[inline]
fn validate_offset<T>(pointer: *const T, memory: &[u8]) -> Result<WasmUsize, TranslationError> {
    let offset = (pointer as usize)
        .checked_sub(memory.as_ptr() as usize)
        .ok_or(TranslationError::OutOfBounds)?;

    let end = offset
        .checked_add(core::mem::size_of::<T>())
        .ok_or(TranslationError::OutOfBounds)?;

    if end > memory.len() {
        return Err(TranslationError::OutOfBounds);
    }

    if (pointer as usize) % core::mem::align_of::<T>() != 0 {
        return Err(TranslationError::Unaligned);
    }

    Ok(offset as WasmUsize)
}

/// Generates both the `WasmPod` impl and the direct wasm-value `as` cast
/// impls from a single type list, so the "types safe to read from arbitrary
/// bytes" set and the "types this crate treats as wasm-native values" set
/// can never drift apart the way the pointer/slice lists just did.
macro_rules! primitive_wasm_pod {
    ($($t:ty),* $(,)?) => {
        $(
            impl sealed::Sealed for $t {}
            impl WasmPod for $t {}

            impl<'a> TranslateFrom<'a, WasmUsize> for $t {
                #[inline]
                unsafe fn translate_from(parameters: WasmUsize, _: &mut [u8]) -> Result<Self, TranslationError> {
                    Ok(parameters as $t)
                }
            }

            impl TranslateInto<WasmUsize> for $t {
                type Output = Self;
                #[inline]
                unsafe fn translate_into(self, _: &mut [u8]) -> Result<Self::Output, TranslationError> {
                    Ok(self as $t)
                }
            }
        )*
    };
}

primitive_wasm_pod!(u8, u16, u32, usize, i8, i16, i32, isize, f32);

#[cfg(target_pointer_width = "64")]
primitive_wasm_pod!(u64, i64, f64);

macro_rules! implicit_pointer_translation {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'a> TranslateFrom<'a, WasmUsize> for *mut $t {
                #[inline]
                unsafe fn translate_from(pointer: WasmUsize, memory: &'a mut [u8]) -> Result<Self, TranslationError> {
                    if pointer == 0 {
                        return Ok(core::ptr::null_mut());
                    }
                    Ok(validate_and_slice::<$t>(pointer as usize, memory)?.as_mut_ptr() as *mut $t)
                }
            }

            impl<'a> TranslateFrom<'a, WasmUsize> for *const $t {
                #[inline]
                unsafe fn translate_from(pointer: WasmUsize, memory: &'a mut [u8]) -> Result<Self, TranslationError> {
                    if pointer == 0 {
                        return Ok(core::ptr::null());
                    }
                    Ok(validate_and_slice::<$t>(pointer as usize, memory)?.as_ptr() as *const $t)
                }
            }

            impl TranslateInto<WasmUsize> for *mut $t {
                type Output = WasmUsize;

                #[inline]
                unsafe fn translate_into(self, memory: &mut [u8]) -> Result<Self::Output, TranslationError> {
                    if self.is_null() {
                        return Ok(0);
                    }
                    validate_offset(self as *const $t, memory)
                }
            }

            impl TranslateInto<WasmUsize> for *const $t {
                type Output = WasmUsize;

                #[inline]
                unsafe fn translate_into(self, memory: &mut [u8]) -> Result<Self::Output, TranslationError> {
                    if self.is_null() {
                        return Ok(0);
                    }
                    validate_offset(self, memory)
                }
            }
        )*
    };
}

// `c_void` belongs *only* here: a raw pointer never requires its pointee
// to hold a valid value, so an opaque type is fine as a pointee but never
// as something read through a reference or slice (see `WasmPod` above,
// and the slice translation below, which is generic over `T: WasmPod`
// specifically to make a `c_void` slice/reference impossible to write).
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

/// Translates a `(buffer, length)` wasm vector into a real `&'a mut [T]`.
/// Generic over `T: WasmPod` instead of a macro-generated type list, so
/// there's no list to accidentally include an unsound type in (this is
/// what the previous `core::ffi::c_void` slice impl was missing).
impl<'a, T: WasmPod> TranslateFrom<'a, WasiVector> for &'a mut [T] {
    #[inline]
    unsafe fn translate_from(
        wasi_vector: WasiVector,
        memory: &'a mut [u8],
    ) -> Result<Self, TranslationError> {
        let start = wasi_vector.buffer as usize;
        let length = wasi_vector.length as usize;

        let slice = validate_and_slice_n::<T>(start, length, memory)?;

        Ok(core::slice::from_raw_parts_mut(
            slice.as_mut_ptr() as *mut T,
            length,
        ))
    }
}

pub struct TranslationSliceIterator<'a, S: WasmPod, D> {
    memory: &'a mut [u8],
    _marker: core::marker::PhantomData<(S, D)>,
}

impl<'a, S: WasmPod, D> TranslateFrom<'a, WasiVector> for TranslationSliceIterator<'a, S, D>
where
    D: Sized + TranslateFrom<'a, S>,
{
    #[inline]
    unsafe fn translate_from(
        wasi_vector: WasiVector,
        data: &'a mut [u8],
    ) -> Result<Self, TranslationError> {
        let start = wasi_vector.buffer as usize;
        let length = wasi_vector.length as usize;

        let slice = validate_and_slice_n::<S>(start, length, data)?;

        Ok(Self {
            memory: slice,
            _marker: core::marker::PhantomData,
        })
    }
}

impl<'a, S: WasmPod, D> Iterator for TranslationSliceIterator<'a, S, D>
where
    D: Sized + TranslateFrom<'a, S>,
{
    type Item = Result<D, TranslationError>;

    fn next(&mut self) -> Option<Self::Item> {
        let elem_size = core::mem::size_of::<S>();
        if self.memory.len() < elem_size {
            return None;
        }

        // Move the real `&'a mut [u8]` out of `self` (leaving an empty
        // slice behind) so splitting it keeps the original `'a` lifetime
        // on both halves, rather than reborrowing through `&mut self`.
        let memory = core::mem::take(&mut self.memory);
        let (current, rest) = memory.split_at_mut(elem_size);
        self.memory = rest;

        // Sound because `S: WasmPod` guarantees every bit pattern of
        // `current`'s bytes is a valid `S`.
        let parameters = unsafe { core::ptr::read_unaligned(current.as_ptr() as *const S) };

        Some(unsafe { D::translate_from(parameters, current) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.memory.len() / core::mem::size_of::<S>();
        (remaining, Some(remaining))
    }
}

impl<'a, S: WasmPod, D> ExactSizeIterator for TranslationSliceIterator<'a, S, D> where
    D: Sized + TranslateFrom<'a, S>
{
}

/// Translates a `WasiVector` of `WasiVector` descriptors — a guest array of
/// `(buffer, length)` pairs, e.g. a WASI iovec array — into a sequence of
/// `&mut [T]`, one per descriptor, each pointing into the *full* linear
/// memory (unlike `TranslationSliceIterator`, whose nested `translate_from`
/// only ever sees one element's own bytes and so can't validate an inner
/// pointer against the wider memory).
///
/// Deliberately **not** `std::iter::Iterator`: each returned slice borrows
/// from `&mut self`, not from `'a`, so the caller cannot hold two yielded
/// slices alive at once. That's what makes this sound — a guest can hand
/// two descriptors overlapping offsets (or one overlapping the descriptor
/// array itself), and this design still never has two live `&mut`
/// references over the same bytes, because only one item can be in scope
/// at a time. Drive it with `while let Some(item) = stream.next() { .. }`.
///
/// If you need every buffer alive *simultaneously* (e.g. building a real
/// `readv`/`writev`-shaped array for a single syscall), this type isn't
/// it — that needs an upfront overlap check across all descriptors before
/// any slice is constructed, which is a different, more involved design.
pub struct TranslationSliceStream<'a, T: WasmPod> {
    descriptors: *const WasiVector,
    remaining: usize,
    memory: *mut [u8],
    _marker: core::marker::PhantomData<(&'a mut [u8], T)>,
}

impl<'a, T: WasmPod> TranslateFrom<'a, WasiVector> for TranslationSliceStream<'a, T> {
    #[inline]
    unsafe fn translate_from(
        wasi_vector: WasiVector,
        memory: &'a mut [u8],
    ) -> Result<Self, TranslationError> {
        let start = wasi_vector.buffer as usize;
        let count = wasi_vector.length as usize;

        // Capture raw access to the *whole* buffer before narrowing it —
        // `validate_and_slice_n` consumes `memory` by value and only hands
        // back the requested sub-range, so this has to happen first via a
        // reborrow (which doesn't move `memory`, unlike a bare cast).
        let memory_ptr: *mut [u8] = &mut *memory as *mut [u8];

        let descriptors = validate_and_slice_n::<WasiVector>(start, count, memory)?;

        Ok(Self {
            descriptors: descriptors.as_ptr() as *const WasiVector,
            remaining: count,
            memory: memory_ptr,
            _marker: core::marker::PhantomData,
        })
    }
}

impl<'a, T: WasmPod> TranslationSliceStream<'a, T> {
    /// Number of descriptors not yet consumed.
    pub fn remaining(&self) -> usize {
        self.remaining
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Result<&mut [T], TranslationError>> {
        if self.remaining == 0 {
            return None;
        }

        // SAFETY: `self.descriptors` was validated at construction to point
        // to `self.remaining` (at the time) valid, in-bounds, aligned
        // `WasiVector` entries; we advance by exactly one entry per call.
        let descriptor = unsafe { core::ptr::read_unaligned(self.descriptors) };
        self.descriptors = unsafe { self.descriptors.add(1) };
        self.remaining -= 1;

        let start = descriptor.buffer as usize;
        let length = descriptor.length as usize;

        // SAFETY: `self.memory` is the full linear memory for this call.
        // This reborrow, and the slice built from it below, are the only
        // live reference into it for the duration of this call — the
        // lending signature (borrowing from `&mut self`) is what prevents
        // the caller from ever holding this alongside a previous or
        // subsequent item.
        let memory: &mut [u8] = unsafe { &mut *self.memory };

        Some(
            validate_and_slice_n::<T>(start, length, memory).map(|slice| unsafe {
                core::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut T, length)
            }),
        )
    }
}
