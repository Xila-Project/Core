use core::{future::Future, mem::transmute, num::NonZeroUsize, pin::Pin};

use alloc::{boxed::Box, string::String};

use crate::Standard;

pub type MainFunctionType = Box<
    dyn Fn(
            Standard,
            String,
        )
            -> Pin<Box<dyn Future<Output = core::result::Result<(), NonZeroUsize>> + 'static>>
        + 'static,
>;

pub struct ReadData {
    main: Option<MainFunctionType>,
}

impl ReadData {
    pub fn new<F>(main: impl Fn(Standard, String) -> F + 'static) -> Self
    where
        F: Future<Output = core::result::Result<(), NonZeroUsize>> + 'static,
    {
        Self {
            main: Some(Box::new(move |standard, arguments| {
                Box::pin(main(standard, arguments))
            })),
        }
    }

    pub const fn new_default() -> [u8; size_of::<Self>()] {
        [0; size_of::<Self>()]
    }

    pub const fn get_size(&self) -> usize {
        size_of::<Self>()
    }

    pub fn get_main(self) -> Option<MainFunctionType> {
        self.main
    }
}

impl TryFrom<&mut [u8]> for &mut ReadData {
    type Error = ();

    fn try_from(value: &mut [u8]) -> core::result::Result<Self, Self::Error> {
        if value.len() != size_of::<ReadData>() {
            return Err(());
        }
        if !(value.as_ptr() as usize).is_multiple_of(core::mem::align_of::<ReadData>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(value.as_mut_ptr()) })
    }
}

impl TryFrom<[u8; size_of::<ReadData>()]> for ReadData {
    type Error = ();

    fn try_from(value: [u8; size_of::<ReadData>()]) -> core::result::Result<Self, Self::Error> {
        Ok(unsafe { transmute::<[u8; size_of::<ReadData>()], Self>(value) })
    }
}

impl AsMut<[u8]> for ReadData {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}
