use core::ffi::c_void;

pub type FunctionPointer = *mut c_void;

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

pub struct FunctionDescriptor {
    pub name: &'static str,
    pub pointer: FunctionPointer,
}
pub trait Registrable {
    fn get_functions(&self) -> &[FunctionDescriptor];

    fn is_xip(&self) -> bool {
        false
    }

    fn get_binary(&self) -> Option<&'static [u8]> {
        None
    }

    fn get_name(&self) -> &'static str;
}
