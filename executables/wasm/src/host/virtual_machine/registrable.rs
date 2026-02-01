use core::ffi::c_void;

pub type FunctionPointer = *mut c_void;

pub struct FunctionDescriptor {
    pub name: &'static str,
    pub pointer: FunctionPointer,
}
pub trait Registrable {
    fn get_functions(&self) -> &[FunctionDescriptor];

    fn is_xip(&self) -> bool {
        false
    }

    fn get_name(&self) -> &'static str;
}
