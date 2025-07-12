#[macro_export]
macro_rules! Mount_static_devices {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), file_system::Error>
    {
        use file_system::create_device;

        $( $Virtual_file_system.mount_static_device($Task_identifier, $Path, create_device!($Device)).await?; )*

        Ok(())
    }()
};

}
