#[macro_export]
macro_rules! mount_static_devices {

    ( $Virtual_file_system:expr, $Task_identifier:expr, &[ $( ($Path:expr, $Device:expr) ),* $(,)? ] ) => {

    async || -> Result<(), $crate::exported_file_system::Error>
    {
        use $crate::exported_file_system::create_device;

        $( $Virtual_file_system.mount_static_device($Task_identifier, $Path, create_device!($Device)).await?; )*

        Ok(())
    }()
};

}
