use alloc::string::String;
use File_system::Device_trait;
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

pub trait Device_executable_trait: Device_trait {
    fn Mount<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Task: Task_identifier_type,
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
        impl Executable::Device_executable_trait for $struct_name {
            fn Mount<'a>(
                Virtual_file_system: &'a Virtual_file_system::Virtual_file_system_type<'a>,
                Task: Task::Task_identifier_type,
            ) -> Result<(), alloc::string::String> {
                use alloc::string::ToString;

                Futures::block_on(Virtual_file_system.Mount_static_device(
                    Task,
                    &$mount_path,
                    File_system::Create_device!($struct_name),
                ))
                .map_err(|Error| Error.to_string())?;

                Ok(())
            }
        }

        impl File_system::Device_trait for $struct_name {
            fn Read(&self, Buffer: &mut [u8]) -> File_system::Result_type<File_system::Size_type> {
                let Read_data: &mut Executable::Read_data_type = Buffer
                    .try_into()
                    .map_err(|_| File_system::Error_type::Invalid_parameter)?;

                *Read_data = Executable::Read_data_type::New($main_function);

                Ok(core::mem::size_of::<Executable::Read_data_type>().into())
            }

            fn Write(&self, _: &[u8]) -> File_system::Result_type<File_system::Size_type> {
                Err(File_system::Error_type::Unsupported_operation)
            }

            fn Get_size(&self) -> File_system::Result_type<File_system::Size_type> {
                Err(File_system::Error_type::Unsupported_operation)
            }

            fn Set_position(
                &self,
                _: &File_system::Position_type,
            ) -> File_system::Result_type<File_system::Size_type> {
                Err(File_system::Error_type::Unsupported_operation)
            }

            fn Flush(&self) -> File_system::Result_type<()> {
                Err(File_system::Error_type::Unsupported_operation)
            }
        }
    };
}

#[macro_export]
macro_rules! Mount_static_executables {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), File_system::Error_type>
    {
        use File_system::{Create_device, Permissions_type};

        $(
            $Virtual_file_system.Mount_static_device($Task_identifier, $Path, Create_device!($Device)).await?;
            $Virtual_file_system.Set_permissions($Path, Permissions_type::Executable ).await?;
        )*

        Ok(())
    }()
};

}
