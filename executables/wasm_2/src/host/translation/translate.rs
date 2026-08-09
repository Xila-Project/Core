use core::ptr::null_mut;

use crate::host::translation::{
    GuestPointer, GuestSlice, WasiVector, WasmPod, WasmPointee, WasmUsize, validate_and_slice,
    validate_and_slice_n,
};

pub trait FromGuest<I>: Sized {
    fn from_guest(&self, memory: &mut [u8]) -> Option<I>;
}

pub trait IntoGuest<O>: Sized {
    fn into_guest(self, memory: &mut [u8]) -> Option<O>;
}
