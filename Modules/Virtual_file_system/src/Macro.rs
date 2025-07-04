#[macro_export]
macro_rules! Mount_static_devices {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), File_system::Error_type>
    {
        use File_system::Create_device;

        $( $Virtual_file_system.Mount_static_device($Task_identifier, $Path, Create_device!($Device)).await?; )*

        Ok(())
    }()
};

}
