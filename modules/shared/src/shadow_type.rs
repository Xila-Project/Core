#[macro_export]
macro_rules! generate_shadow_type {
    ($proxy_name:ident, $real_type:ty, $size:expr, $align:expr) => {
        /// cbindgen:field-names=[data]
        #[repr(C)]
        #[repr(align($align))]
        pub struct $proxy_name {
            pub data: [u8; $size],
        }

        const _: () = {
            unsafe {
                assert!(::core::mem::size_of::<$real_type>() == $size);
                assert!(::core::mem::align_of::<$real_type>() == $align);
                assert!(::core::mem::size_of::<$proxy_name>() == $size);
                assert!(::core::mem::align_of::<$proxy_name>() == $align);
            }
        };

        impl $proxy_name {
            #[inline]
            pub fn from_real(real: $real_type) -> Self {
                unsafe {
                    let mut proxy = ::core::mem::MaybeUninit::<Self>::uninit();
                    ::core::ptr::write(proxy.as_mut_ptr().cast::<$real_type>(), real);
                    proxy.assume_init()
                }
            }

            /// # Safety
            /// `self` must have been initialized via `from_real`.
            #[inline]
            pub unsafe fn into_real(self) -> $real_type {
                unsafe {
                    let real = ::core::ptr::read(self.data.as_ptr().cast::<$real_type>());
                    ::core::mem::forget(self);
                    real
                }
            }

            /// # Safety
            /// `self` must have been initialized via `from_real`.
            #[inline]
            pub unsafe fn as_real(&self) -> &$real_type {
                unsafe { &*self.data.as_ptr().cast::<$real_type>() }
            }

            /// # Safety
            /// `self` must have been initialized via `from_real`.
            #[inline]
            pub unsafe fn as_real_mut(&mut self) -> &mut $real_type {
                unsafe { &mut *self.data.as_mut_ptr().cast::<$real_type>() }
            }
        }

        impl ::core::ops::Deref for $proxy_name {
            type Target = $real_type;
            #[inline]
            fn deref(&self) -> &$real_type {
                // SAFETY: from_real is the only safe constructor, guarantees valid init.
                // Size and align are verified by the const assertions above.
                unsafe { &*self.data.as_ptr().cast::<$real_type>() }
            }
        }

        impl ::core::ops::DerefMut for $proxy_name {
            #[inline]
            fn deref_mut(&mut self) -> &mut $real_type {
                unsafe { &mut *self.data.as_mut_ptr().cast::<$real_type>() }
            }
        }
    };
}
