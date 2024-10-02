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
        Create_device, Create_file_system, Device_type, File_type, Flags_type, LittleFS, Mode_type,
        Open_type, Path_type, Position_type, Result_type, Status_type, Tests::Memory_device_type,
    };

    let Task_instance = Task::Initialize().expect("Failed to initialize task manager");

    Users::Initialize().expect("Failed to initialize users manager");

    Time::Initialize(Box::new(Drivers::Native::Time_driver_type::New()))
        .expect("Failed to initialize time manager");

    let Task = Task_instance
        .Get_current_task_identifier()
        .expect("Failed to get current task identifier");

    let Device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    let Cache_size = 256;

    LittleFS::File_system_type::Format(Device.clone(), Cache_size).unwrap();
    let File_system = LittleFS::File_system_type::New(Device, Cache_size).unwrap();

    let Virtual_file_system = File_system::Initialize(Create_file_system!(File_system)).unwrap();

    let File_path = "/file";

    let File = File_type::Open(
        Virtual_file_system,
        File_path,
        Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None),
        Task,
    )
    .unwrap();

    let Data = b"Hello, world!";

    File.Write(Data).unwrap();

    File.Set_position(&Position_type::Start(0)).unwrap();

    let mut Buffer = [0; 13];

    File.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);

    std::mem::drop(File);

    Virtual_file_system.Remove(File_path).unwrap();

    let (Pipe_read, Pipe_write) = File_type::Create_unnamed_pipe(
        Virtual_file_system,
        512_usize.into(),
        Status_type::default(),
        Task,
    )
    .unwrap();

    Pipe_write.Write(Data).unwrap();

    let mut Buffer = [0; 13];

    Pipe_read.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);

    let Pipe_path = "/pipe";

    Virtual_file_system
        .Create_named_pipe(&Pipe_path, 512, Task)
        .unwrap();

    let Pipe_read = File_type::Open(
        Virtual_file_system,
        Pipe_path,
        Mode_type::Read_only.into(),
        Task,
    )
    .unwrap();

    let Pipe_write = File_type::Open(
        Virtual_file_system,
        Pipe_path,
        Mode_type::Write_only.into(),
        Task,
    )
    .unwrap();

    Pipe_write.Write(Data).unwrap();

    let mut Buffer = [0; 13];
    Pipe_read.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);

    std::mem::drop(Pipe_read);
    std::mem::drop(Pipe_write);

    let Device_file = File_type::Open(
        Virtual_file_system,
        Device_path,
        Mode_type::Read_write.into(),
        Task,
    )
    .unwrap();

    let Data = 0x1234567890ABCDEF_u64;

    Device_file.Write(&Data.to_le_bytes()).unwrap();

    Device_file.Set_position(&Position_type::Start(0)).unwrap();

    let mut Buffer = [0; 8];

    Device_file.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, Data.to_le_bytes());

    std::mem::drop(Device_file);

    Virtual_file_system.Remove(Device_path).unwrap();
}
