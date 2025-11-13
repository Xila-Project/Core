use core::any::Any;

use alloc::boxed::Box;

pub struct Context {
    private_data: Option<Box<dyn Any>>,
}

impl Context {
    pub fn new<T: 'static>(private_data: Option<T>) -> Self {
        Self {
            private_data: private_data.map(|data| Box::new(data) as Box<dyn Any>),
        }
    }

    pub fn get_private_data(&mut self) -> Option<&mut Box<dyn Any>> {
        self.private_data.as_mut()
    }

    pub fn get_private_data_of_type<T: 'static>(&mut self) -> Option<&mut T> {
        self.private_data.as_mut()?.downcast_mut::<T>()
    }

    pub fn take_private_data(&mut self) -> Option<Box<dyn Any>> {
        self.private_data.take()
    }

    pub fn take_private_data_of_type<T: 'static>(&mut self) -> Option<Box<T>> {
        if self.private_data.as_ref()?.is::<T>() {
            let data = self.private_data.take()?;
            Some(data.downcast::<T>().ok()?)
        } else {
            None
        }
    }

    pub fn set_private_data(&mut self, data: Box<dyn Any>) {
        self.private_data = Some(data);
    }
}
