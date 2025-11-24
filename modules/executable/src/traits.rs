use crate::Standard;
use alloc::{boxed::Box, string::String, vec::Vec};
use core::{num::NonZeroUsize, pin::Pin};
use file_system::{
    ControlCommand, ControlDirectionFlags, DirectBaseOperations, DirectCharacterDevice,
    MountOperations,
};

pub const GET_MAIN_FUNCTION: ControlCommand =
    ControlCommand::new::<MainFunction>(ControlDirectionFlags::Read, b'E', 1);

pub trait ExecutableTrait: 'static + Send + Sync {
    fn main(standard: Standard, arguments: Vec<String>) -> MainFuture;
}

pub type MainFuture =
    Pin<Box<dyn Future<Output = core::result::Result<(), NonZeroUsize>> + 'static>>;

pub type MainFunction = Option<Box<dyn Fn(Standard, Vec<String>) -> MainFuture + 'static>>;

pub struct ExecutableWrapper<T: ExecutableTrait>(T);

impl<T: ExecutableTrait> ExecutableWrapper<T> {
    pub fn new(executable: T) -> Self {
        Self(executable)
    }
}

impl<T: ExecutableTrait> MountOperations for ExecutableWrapper<T> {}

impl<T: ExecutableTrait> DirectBaseOperations for ExecutableWrapper<T> {
    fn read(
        &self,
        _buffer: &mut [u8],
        _absolute_position: file_system::Size,
    ) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn write(
        &self,
        _buffer: &[u8],
        _absolute_position: file_system::Size,
    ) -> file_system::Result<usize> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn control(
        &self,
        command: ControlCommand,
        argument: &mut file_system::ControlArgument,
    ) -> file_system::Result<()> {
        log::debug!("ExecutableWrapper control command: {:?}", command);

        match command {
            GET_MAIN_FUNCTION => {
                *argument
                    .cast::<MainFunction>()
                    .ok_or(file_system::Error::InvalidParameter)? = Some(Box::new(T::main));
                Ok(())
            }
            _ => Err(file_system::Error::UnsupportedOperation),
        }
    }
}

impl<T: ExecutableTrait> DirectCharacterDevice for ExecutableWrapper<T> {}

#[macro_export]
macro_rules! mount_executables {

    ( $virtual_file_system:expr, $task:expr, &[ $( ($path:expr, $executable:expr) ),* $(,)? ] ) => {

    async || -> $crate::exported_virtual_file_system::Result<()>
    {
        use $crate::exported_file_system::{Permissions};
        use $crate::exported_virtual_file_system::ItemStatic;

        $(
            let __executable = $crate::ExecutableWrapper::new($executable);

            let _ = $virtual_file_system.remove($task, $path).await;
            $virtual_file_system.mount_character_device($task, $path, __executable).await?;
            $virtual_file_system.set_permissions($task, $path, Permissions::EXECUTABLE).await?;
        )*

        Ok(())
    }()
};

}
