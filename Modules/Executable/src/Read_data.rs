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
    Main: Option<Main_function_type>,
    Stack_size: usize,
}

impl Read_data_type {
    pub fn New<F>(Main: impl Fn(Standard_type, String) -> F + 'static, Stack_size: usize) -> Self
    where
        F: Future<Output = Result<(), NonZeroUsize>> + 'static,
    {
        Self {
            Main: Some(Box::new(move |Standard, Arguments| {
                Box::pin(Main(Standard, Arguments))
            })),

            Stack_size,
        }
    }

    pub const fn New_default() -> [u8; size_of::<Self>()] {
        [0; size_of::<Self>()]
    }

    pub const fn Get_size(&self) -> usize {
        size_of::<Self>()
    }

    pub fn Get_main(self) -> Option<Main_function_type> {
        self.Main
    }

    pub fn Get_stack_size(&self) -> usize {
        self.Stack_size
    }
}

impl TryFrom<&mut [u8]> for &mut Read_data_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Read_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Read_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(Value.as_mut_ptr()) })
    }
}

impl TryFrom<[u8; size_of::<Read_data_type>()]> for Read_data_type {
    type Error = ();

    fn try_from(Value: [u8; size_of::<Read_data_type>()]) -> Result<Self, Self::Error> {
        Ok(unsafe { transmute::<[u8; size_of::<Read_data_type>()], Self>(Value) })
    }
}

impl AsMut<[u8]> for Read_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}
