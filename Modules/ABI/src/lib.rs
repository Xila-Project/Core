#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::c_void;

type Function_pointer = *mut c_void;

pub struct Function_descriptor_type {
    pub Name: &'static str,
    pub Function_pointer: Function_pointer,
}

pub trait ABI_trait {
    fn Get_functions(&self) -> Vec<Function_descriptor_type>;
}
