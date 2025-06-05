#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate alloc;

mod Device;
mod Directory;
mod Error;
mod File;
mod File_system;
mod Hierarchy;
mod Pipe;
mod Socket;

pub use Directory::*;
pub use Error::*;
pub use File::*;
pub use File_system::*;
pub use Hierarchy::*;
pub use Socket::Socket_address_type;

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
