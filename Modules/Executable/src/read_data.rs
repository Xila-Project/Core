use core::{future::Future, mem::transmute, num::NonZeroUsize, pin::Pin};

use alloc::{boxed::Box, string::String};

use crate::Standard_type;

pub type Main_function_type = Box<
    dyn Fn(
            Standard_type,
            String,
        ) -> Pin<Box<dyn Future<Output = Result<(), NonZeroUsize>> + 'static>>
        + 'static,
>;

pub struct Read_data_type {
    main: Option<Main_function_type>,
}

impl Read_data_type {
    pub fn new<F>(main: impl Fn(Standard_type, String) -> F + 'static) -> Self
    where
        F: Future<Output = Result<(), NonZeroUsize>> + 'static,
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

    pub fn get_main(self) -> Option<Main_function_type> {
        self.main
    }
}

impl TryFrom<&mut [u8]> for &mut Read_data_type {
    type Error = ();

    fn try_from(value: &mut [u8]) -> Result<Self, Self::Error> {
        if value.len() != size_of::<Read_data_type>() {
            return Err(());
        }
        if !(value.as_ptr() as usize).is_multiple_of(core::mem::align_of::<Read_data_type>()) {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(value.as_mut_ptr()) })
    }
}

impl TryFrom<[u8; size_of::<Read_data_type>()]> for Read_data_type {
    type Error = ();

    fn try_from(value: [u8; size_of::<Read_data_type>()]) -> Result<Self, Self::Error> {
        Ok(unsafe { transmute::<[u8; size_of::<Read_data_type>()], Self>(value) })
    }
}

impl AsMut<[u8]> for Read_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}
