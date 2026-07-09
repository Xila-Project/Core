use core::ffi::c_void;

pub type FunctionPointer = *mut c_void;

pub struct FunctionDescriptor {
    pub name: &'static str,
    pub pointer: FunctionPointer,
}

#[macro_export]
macro_rules! define_wasi_module {
    (
        module: $mod_name:expr;
        $(
            fn $fn_name:ident ( $($arg:ident : $arg_ty:ty),* $(,)? ) -> $ret:ty $body:block
        )*
    ) => {
        // 1. Generate the actual functions
        $(
            pub fn $fn_name ( $($arg : $arg_ty),* ) -> $ret $body
        )*

        // 2. Generate the linker registration function automatically
        pub fn add_to_linker(linker: &mut wasmi::Linker<$crate::host::store::GlobalStore>) -> $crate::host::error::Result<()> {
            $(
                linker.func_wrap($mod_name, stringify!($fn_name), $fn_name)?;
            )*
            Ok(())
        }
    };
}
