use std::ffi::CString;

use super::Native_symbol_type;

pub struct Symbol_type {
    Name: CString,
    Signature: CString,
    Symbol: Native_symbol_type,
}

impl Symbol_type {
    pub fn New(Name: &str, Signature: &str, Function: impl Fn()) -> Self {
        unsafe {
            Self {
                Name: std::ffi::CString::new(Name).unwrap(),
                Signature: std::ffi::CString::new(Signature).unwrap(),
                Symbol: Native_symbol_type {
                    symbol: Name.as_ptr() as *const i8,
                    func_ptr: std::mem::transmute(&Function), // TODO: Check if this is correct
                    signature: Signature.as_ptr() as *const i8,
                    attachment: std::ptr::null_mut(),
                },
            }
        }
    }

    pub fn Get_name(&self) -> &str {
        self.Name.to_str().unwrap()
    }

    pub fn Get_signature(&self) -> &str {
        self.Signature.to_str().unwrap()
    }
}

