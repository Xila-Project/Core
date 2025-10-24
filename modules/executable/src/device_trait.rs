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
        impl $crate::DeviceExecutableTrait for $struct_name {
            fn mount<'a>(
                virtual_file_system: &'a $crate::exported_virtual_file_system::VirtualFileSystem<
                    'a,
                >,
                task: $crate::exported_task::TaskIdentifier,
            ) -> core::result::Result<(), alloc::string::String> {
                use alloc::string::ToString;

                $crate::exported_futures::block_on(virtual_file_system.mount_static_device(
                    task,
                    &$mount_path,
                    $crate::exported_file_system::create_device!($struct_name),
                ))
                .map_err(|error| error.to_string())?;

                Ok(())
            }
        }

        impl $crate::exported_file_system::DeviceTrait for $struct_name {
            fn read(
                &self,
                buffer: &mut [u8],
            ) -> $crate::exported_file_system::Result<$crate::exported_file_system::Size> {
                let read_data: &mut $crate::ReadData = buffer
                    .try_into()
                    .map_err(|_| $crate::exported_file_system::Error::InvalidParameter)?;

                *read_data = $crate::ReadData::new($main_function);

                Ok(core::mem::size_of::<$crate::ReadData>().into())
            }

            fn write(
                &self,
                _: &[u8],
            ) -> $crate::exported_file_system::Result<$crate::exported_file_system::Size> {
                Err($crate::exported_file_system::Error::UnsupportedOperation)
            }

            fn get_size(
                &self,
            ) -> $crate::exported_file_system::Result<$crate::exported_file_system::Size> {
                Err($crate::exported_file_system::Error::UnsupportedOperation)
            }

            fn set_position(
                &self,
                _: &$crate::exported_file_system::Position,
            ) -> $crate::exported_file_system::Result<$crate::exported_file_system::Size> {
                Err($crate::exported_file_system::Error::UnsupportedOperation)
            }

            fn flush(&self) -> $crate::exported_file_system::Result<()> {
                Err($crate::exported_file_system::Error::UnsupportedOperation)
            }
        }
    };
}

#[macro_export]
macro_rules! mount_static_executables {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), $crate::exported_file_system::Error>
    {
        use $crate::exported_file_system::{create_device, Permissions};

        $(
            $Virtual_file_system.mount_static_device($Task_identifier, $Path, create_device!($Device)).await?;
            $Virtual_file_system.set_permissions($Path, Permissions::EXECUTABLE ).await?;
        )*

        Ok(())
    }()
};

}
