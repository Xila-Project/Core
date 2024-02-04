use super::{Module_instance_type, Pointer_type};

use std::ffi::c_void;

pub struct Allocation_type<'a> {
    Module_instance: &'a Module_instance_type,
    Native_pointer: *mut c_void,
    Pointer: Pointer_type,
    Size: usize,
}

impl<'a> Allocation_type<'a> {
    pub fn New(
        Module_instance: &'a Module_instance_type,
        Native_pointer: *mut c_void,
        Pointer: Pointer_type,
        Size: usize,
    ) -> Allocation_type {
        Allocation_type {
            Module_instance,
            Native_pointer,
            Pointer,
            Size,
        }
    }

    pub fn Get_pointer(&self) -> Pointer_type {
        self.Pointer
    }

    pub fn Get_size(&self) -> usize {
        self.Size
    }
}
