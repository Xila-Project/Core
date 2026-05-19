#[macro_export]
macro_rules! generate_shadow_type {
    ($proxy_name:ident, $real_type:ty, $size:expr, $align:expr) => {
        /// cbindgen:field-names=[_align, bindgen_opaque_buffer]
        #[repr(C)]
        pub struct $proxy_name {
            // Dynamically computed array size matching the architecture footprint.
            pub bindgen_opaque_buffer: [u8; ::core::mem::size_of::<$real_type>()],
        }

        impl $proxy_name {
            #[inline]
            pub fn from_real(real: $real_type) -> Self {
                unsafe {
                    let mut proxy = ::core::mem::MaybeUninit::<Self>::uninit();
                    ::core::ptr::write(proxy.as_mut_ptr().cast::<$real_type>(), real);
                    proxy.assume_init()
                }
            }

            #[inline]
            pub fn into_real(self) -> $real_type {
                unsafe {
                    let real = ::core::ptr::read((&self as *const Self).cast::<$real_type>());
                    ::core::mem::forget(self);
                    real
                }
            }
        }

        impl ::core::ops::Deref for $proxy_name {
            type Target = $real_type;
            #[inline]
            fn deref(&self) -> &$real_type {
                unsafe { &*(self.bindgen_opaque_buffer.as_ptr().cast::<$real_type>()) }
            }
        }

        impl ::core::ops::DerefMut for $proxy_name {
            #[inline]
            fn deref_mut(&mut self) -> &mut $real_type {
                unsafe { &mut *(self.bindgen_opaque_buffer.as_mut_ptr().cast::<$real_type>()) }
            }
        }
    };
}
