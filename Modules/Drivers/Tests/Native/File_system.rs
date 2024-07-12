#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use File_system::Path_type;

const Device_path: &Path_type = unsafe { Path_type::New_unchecked_constant("/Device") };

#[cfg(target_os = "linux")]
#[test]
fn Test_file_system() {
    use std::sync::RwLock;

    use File_system::{
        Device_trait, File_type, Mode_type, Path_type, Position_type, Result_type, Status_type,
    };

    Task::Initialize().expect("Failed to initialize task manager");

    Users::Initialize().expect("Failed to initialize users manager");

    let File_system =
        Drivers::Native::File_system_type::New().expect("Failed to create file system");

    let Virtual_file_system = File_system::Initialize().expect("Failed to initialize file system");

    Virtual_file_system
        .Mount(Box::new(File_system), Path_type::Get_root())
        .expect("Failed to mount file system");

    let File_path = Path_type::New("/test.txt").expect("Failed to create path");

    if Virtual_file_system
        .Exists(File_path)
        .expect("Failed to check if file exists")
    {
        Virtual_file_system
            .Delete(File_path, false)
            .expect("Failed to delete file");
    }

    Virtual_file_system
        .Create_file(File_path)
        .expect("Failed to create file");

    let File = File_type::Open(
        Virtual_file_system,
        File_path,
        Mode_type::Read_write().into(),
    )
    .expect("Failed to open file");

    let Data = b"Hello, world!";

    File.Write(Data).expect("Failed to write data");

    File.Set_position(&Position_type::Start(0))
        .expect("Failed to set position");

    let mut Buffer = [0; 13];

    File.Read(&mut Buffer).expect("Failed to read data");

    assert_eq!(Buffer, *Data);

    std::mem::drop(File);

    Virtual_file_system
        .Delete(File_path, false)
        .expect("Failed to delete file");

    let (Pipe_read, Pipe_write) = File_type::Create_unnamed_pipe(
        Virtual_file_system,
        512_usize.into(),
        Status_type::default(),
    )
    .expect("Failed to create pipe");

    Pipe_write.Write(Data).expect("Failed to write data");

    let mut Buffer = [0; 13];

    Pipe_read.Read(&mut Buffer).expect("Failed to read data");

    assert_eq!(Buffer, *Data);

    let Pipe_path = Path_type::New("/pipe").expect("Failed to create path");

    if Virtual_file_system
        .Exists(Pipe_path)
        .expect("Failed to check if pipe exists")
    {
        Virtual_file_system
            .Delete(Pipe_path, false)
            .expect("Failed to delete pipe");
    }

    Virtual_file_system
        .Create_named_pipe(&Pipe_path, 512_usize.into())
        .expect("Failed to create pipe");

    let Pipe_read = File_type::Open(
        Virtual_file_system,
        Pipe_path,
        Mode_type::Read_only().into(),
    )
    .expect("Failed to open pipe");

    let Pipe_write = File_type::Open(
        Virtual_file_system,
        Pipe_path,
        Mode_type::Write_only().into(),
    )
    .expect("Failed to open pipe");

    let mut Buffer = [0; 13];

    Pipe_write.Write(Data).expect("Failed to write data");

    Pipe_read.Read(&mut Buffer).expect("Failed to read data");

    assert_eq!(Buffer, *Data);

    std::mem::drop(Pipe_read);
    std::mem::drop(Pipe_write);

    Virtual_file_system
        .Delete(Pipe_path, false)
        .expect("Failed to delete pipe");

    struct Dummy_device_type(RwLock<u64>);

    impl Device_trait for Dummy_device_type {
        fn Read(&self, Buffer: &mut [u8]) -> Result_type<usize> {
            Buffer.copy_from_slice(&self.0.read()?.to_le_bytes());

            Ok(std::mem::size_of::<u64>())
        }

        fn Write(&self, Buffer: &[u8]) -> Result_type<usize> {
            *self.0.write()? = u64::from_le_bytes(Buffer.try_into().unwrap());

            Ok(std::mem::size_of::<u64>())
        }

        fn Get_size(&self) -> Result_type<usize> {
            Ok(std::mem::size_of::<u64>())
        }

        fn Set_position(&self, _: &Position_type) -> Result_type<usize> {
            Ok(0)
        }

        fn Flush(&self) -> Result_type<()> {
            Ok(())
        }
    }

    let Device = Dummy_device_type(RwLock::new(0));

    Virtual_file_system
        .Add_device(&Device_path, Box::new(Device))
        .expect("Failed to add device");

    let Device_file = File_type::Open(
        Virtual_file_system,
        Device_path,
        Mode_type::Read_write().into(),
    )
    .expect("Failed to open device");

    let Data = 0x1234567890ABCDEF_u64;

    Device_file
        .Write(&Data.to_le_bytes())
        .expect("Failed to write data");

    Device_file
        .Set_position(&Position_type::Start(0))
        .expect("Failed to set position");

    let mut Buffer = [0; 8];

    Device_file.Read(&mut Buffer).expect("Failed to read data");

    assert_eq!(Buffer, Data.to_le_bytes());

    std::mem::drop(Device_file);

    Virtual_file_system
        .Delete(Device_path, false)
        .expect("Failed to delete device");
}
