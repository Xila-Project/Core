#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_void;

pub type Function_pointer = *mut c_void;

#[macro_export]
macro_rules! Function_descriptor {
    ($Function:ident) => {
        $crate::Function_descriptor_type {
            Name: stringify!($Function),
            Pointer: $Function as *mut std::ffi::c_void,
        }
    };
}

#[macro_export]
macro_rules! Function_descriptors {
    ($($Function:ident),*) => {
        [$(
            $crate::Function_descriptor!($Function),
        )*]
    };
}

pub struct Function_descriptor_type {
    pub Name: &'static str,
    pub Pointer: Function_pointer,
}
pub trait Registrable_trait {
    fn Get_functions(&self) -> &[Function_descriptor_type];

    fn Is_XIP(&self) -> bool {
        false
    }

    fn Get_binary(&self) -> Option<&'static [u8]> {
        None
    }

    fn Get_name(&self) -> &'static str;
}
