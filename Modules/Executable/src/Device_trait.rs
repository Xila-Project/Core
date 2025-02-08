use File_system::Device_trait;
use Task::Task_identifier_type;
use Virtual_file_system::Virtual_file_system_type;

pub trait Device_executable_trait: Device_trait {
    fn Mount<'a>(
        Virtual_file_system: &'a Virtual_file_system_type<'a>,
        Task: Task_identifier_type,
    ) -> Result<(), String>;
}

#[macro_export]
macro_rules! Mount_static_executables {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    || -> Result<(), File_system::Error_type>
    {
        use File_system::Create_device;

        $(
            $Virtual_file_system.Mount_static_device($Task_identifier, $Path, Create_device!($Device))?;
            $Virtual_file_system.Set_permissions($Task_identifier, $Path, Permissions_type::Executable )?;
        )*



        Ok(())
    }()
};

}
