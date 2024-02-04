use std::ffi::CString;
use wamr_sys::NativeSymbol;

pub struct Symbol_type {
    Name: CString,
    Signature: CString,
    Symbol: NativeSymbol,
}

impl Symbol_type {
    pub fn New(Name: &str, Signature: &str, Function: impl Fn()) -> Self {
        unsafe {
            Self {
                Name: std::ffi::CString::new(Name).unwrap(),
                Signature: std::ffi::CString::new(Signature).unwrap(),
                Symbol: NativeSymbol {
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

#[macro_export]
macro_rules! Declare_native_symbol {
    ($name:ident, $signature:expr) => {
        mod Symbols {
            use paste::paste;
            use wamr_sys::NativeSymbol;
            use std::ptr::null_mut;
            paste!{
                const $name: NativeSymbol = NativeSymbol {
                    symbol: concat!(stringify!($name), "\0").as_ptr() as *const i8,
                    func_ptr: super::$name as *mut _,
                    signature: concat!($signature, "\0").as_ptr() as *const i8,
                    attachment: null_mut(),
                };
            }
        }
    }
}