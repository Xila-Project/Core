use alloc::string::String;
use file_system::DeviceTrait;
use task::TaskIdentifier;
use virtual_file_system::VirtualFileSystem;

pub trait DeviceExecutableTrait: DeviceTrait {
    fn mount<'a>(
        virtual_file_system: &'a VirtualFileSystem<'a>,
        task: TaskIdentifier,
    ) -> Result<(), String>;
}

/// Macro to automatically implement DeviceExecutableTrait for a struct with a main function.
///
/// This macro generates both the DeviceExecutableTrait implementation and the DeviceTrait
/// implementation for executable devices.
///
/// # Usage
///
/// For simple executables:
/// ```rust
/// extern crate alloc;
///
/// pub struct MyExecutableType;
///
/// async fn my_main_function(
///     standard: executable::Standard,
///     arguments: String
/// ) -> Result<(), core::num::NonZeroUsize> {
///    standard.print_line(&arguments);
///
///    Ok(())
/// }
///
/// executable::implement_executable_device!(
///     Structure: MyExecutableType,
///     Mount_path: "/binaries/MyExecutable",
///     Main_function: my_main_function,
/// );
/// ```
#[macro_export]
macro_rules! implement_executable_device {
    // Simple executable without constructor
    (
        Structure: $struct_name:ident,
        Mount_path: $mount_path:expr,
        Main_function: $main_function:path,
    ) => {
        impl executable::DeviceExecutableTrait for $struct_name {
            fn mount<'a>(
                virtual_file_system: &'a virtual_file_system::VirtualFileSystem<'a>,
                task: task::TaskIdentifier,
            ) -> core::result::Result<(), alloc::string::String> {
                use alloc::string::ToString;

                futures::block_on(virtual_file_system.mount_static_device(
                    task,
                    &$mount_path,
                    file_system::create_device!($struct_name),
                ))
                .map_err(|error| error.to_string())?;

                Ok(())
            }
        }

        impl file_system::DeviceTrait for $struct_name {
            fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
                let read_data: &mut executable::ReadData = buffer
                    .try_into()
                    .map_err(|_| file_system::Error::InvalidParameter)?;

                *read_data = executable::ReadData::new($main_function);

                Ok(core::mem::size_of::<executable::ReadData>().into())
            }

            fn write(&self, _: &[u8]) -> file_system::Result<file_system::Size> {
                Err(file_system::Error::UnsupportedOperation)
            }

            fn get_size(&self) -> file_system::Result<file_system::Size> {
                Err(file_system::Error::UnsupportedOperation)
            }

            fn set_position(
                &self,
                _: &file_system::Position,
            ) -> file_system::Result<file_system::Size> {
                Err(file_system::Error::UnsupportedOperation)
            }

            fn flush(&self) -> file_system::Result<()> {
                Err(file_system::Error::UnsupportedOperation)
            }
        }
    };
}

#[macro_export]
macro_rules! mount_static_executables {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), file_system::Error>
    {
        use file_system::{create_device, Permissions};

        $(
            $Virtual_file_system.mount_static_device($Task_identifier, $Path, create_device!($Device)).await?;
            $Virtual_file_system.set_permissions($Path, Permissions::EXECUTABLE ).await?;
        )*

        Ok(())
    }()
};

}
