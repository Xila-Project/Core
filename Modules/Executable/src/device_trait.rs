use alloc::string::String;
use file_system::Device_trait;
use task::Task_identifier_type;
use virtual_file_system::Virtual_file_system_type;

pub trait Device_executable_trait: Device_trait {
    fn mount<'a>(
        virtual_file_system: &'a Virtual_file_system_type<'a>,
        task: Task_identifier_type,
    ) -> Result<(), String>;
}

/// Macro to automatically implement Device_executable_trait for a struct with a main function.
///
/// This macro generates both the Device_executable_trait implementation and the Device_trait
/// implementation for executable devices.
///
/// # Usage
///
/// For simple executables:
/// ```rust
/// Implement_executable_device!(
///     Structure: MyExecutableType,
///     Mount_path: "/Binaries/MyExecutable",
///     Main_function: my_main_function,
/// );
/// ```
#[macro_export]
macro_rules! Implement_executable_device {
    // Simple executable without constructor
    (
        Structure: $struct_name:ident,
        Mount_path: $mount_path:expr,
        Main_function: $main_function:path,
    ) => {
        impl executable::Device_executable_trait for $struct_name {
            fn mount<'a>(
                virtual_file_system: &'a virtual_file_system::Virtual_file_system_type<'a>,
                task: task::Task_identifier_type,
            ) -> Result<(), alloc::string::String> {
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

        impl file_system::Device_trait for $struct_name {
            fn read(&self, buffer: &mut [u8]) -> file_system::Result_type<file_system::Size_type> {
                let read_data: &mut executable::Read_data_type = buffer
                    .try_into()
                    .map_err(|_| file_system::Error_type::Invalid_parameter)?;

                *read_data = executable::Read_data_type::new($main_function);

                Ok(core::mem::size_of::<executable::Read_data_type>().into())
            }

            fn write(&self, _: &[u8]) -> file_system::Result_type<file_system::Size_type> {
                Err(file_system::Error_type::Unsupported_operation)
            }

            fn get_size(&self) -> file_system::Result_type<file_system::Size_type> {
                Err(file_system::Error_type::Unsupported_operation)
            }

            fn set_position(
                &self,
                _: &file_system::Position_type,
            ) -> file_system::Result_type<file_system::Size_type> {
                Err(file_system::Error_type::Unsupported_operation)
            }

            fn flush(&self) -> file_system::Result_type<()> {
                Err(file_system::Error_type::Unsupported_operation)
            }
        }
    };
}

#[macro_export]
macro_rules! Mount_static_executables {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), file_system::Error_type>
    {
        use file_system::{create_device, Permissions_type};

        $(
            $Virtual_file_system.mount_static_device($Task_identifier, $Path, create_device!($Device)).await?;
            $Virtual_file_system.set_permissions($Path, Permissions_type::EXECUTABLE ).await?;
        )*

        Ok(())
    }()
};

}
