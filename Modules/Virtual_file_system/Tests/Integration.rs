#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use Task::Task_identifier_type;

use File_system::{
    Create_device, Create_file_system, Flags_type, Memory_device_type, Mode_type, Open_type,
    Path_type, Position_type, Status_type,
};
#[cfg(target_os = "linux")]
use Task::Test;
use Virtual_file_system::{File_type, Virtual_file_system_type};

async fn Initialize<'a>() -> (Task_identifier_type, Virtual_file_system_type<'a>) {
    let Task_instance = Task::Initialize();

    unsafe {
        let _ = Task_instance.Register_task().await;
    }

    let _ = Users::Initialize();

    let _ = Time::Initialize(Create_device!(Drivers::Native::Time_driver_type::New()));

    let Task = Task_instance
        .Get_current_task_identifier()
        .expect("Failed to get current task identifier");

    let Device = Create_device!(Memory_device_type::<512>::New(1024 * 512));

    let Cache_size = 256;

    LittleFS::File_system_type::Format(Device.clone(), Cache_size).unwrap();
    let File_system = LittleFS::File_system_type::New(Device, Cache_size).unwrap();

    let Virtual_file_system = Virtual_file_system_type::New(
        Task_instance,
        Users::Get_instance(),
        Time::Get_instance(),
        Create_file_system!(File_system),
        None,
    ).await
    .unwrap();

    (Task, Virtual_file_system)
}

#[cfg(target_os = "linux")]
#[Test]
async fn Test_file() {
    let (_, Virtual_file_system) = Initialize();

    let File_path = "/file";

    let File = File_type::Open(
        &Virtual_file_system,
        File_path,
        Flags_type::New(Mode_type::Read_write, Some(Open_type::Create_only), None),
    )
    .unwrap();

    let Data = b"Hello, world!";

    File.Write(Data).unwrap();

    File.Set_position(&Position_type::Start(0)).unwrap();

    let mut Buffer = [0; 13];

    File.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);

    core::mem::drop(File);

    Virtual_file_system.Remove(File_path).unwrap();
}

#[cfg(target_os = "linux")]
#[Test]
async fn Test_unnamed_pipe() {
    let (Task, Virtual_file_system) = Initialize();

    let (Pipe_read, Pipe_write) =
        File_type::Create_unnamed_pipe(&Virtual_file_system, 512, Status_type::default(), Task)
            .unwrap();

    let Data = b"Hello, world!";

    Pipe_write.Write(Data).unwrap();

    let mut Buffer = [0; 13];

    Pipe_read.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);
}

#[cfg(target_os = "linux")]
#[Test]
async fn Test_named_pipe() {
    let (Task, Virtual_file_system) = Initialize();

    let Pipe_path = "/pipe";

    Virtual_file_system
        .Create_named_pipe(&Pipe_path, 512, Task)
        .unwrap();

    let Pipe_read =
        File_type::Open(&Virtual_file_system, Pipe_path, Mode_type::Read_only.into()).unwrap();

    let Pipe_write = File_type::Open(
        &Virtual_file_system,
        Pipe_path,
        Mode_type::Write_only.into(),
    )
    .unwrap();

    let Data = b"Hello, world!";

    Pipe_write.Write(Data).unwrap();

    let mut Buffer = [0; 13];
    Pipe_read.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);

    core::mem::drop(Pipe_read);
    core::mem::drop(Pipe_write);

    Virtual_file_system.Remove(Pipe_path).unwrap();
}

#[cfg(target_os = "linux")]
#[Test]
fn Test_device() {
    let (Task, Virtual_file_system) = Initialize();

    const Device_path: &Path_type = Path_type::From_str("/Device");

    let Device = Create_device!(Memory_device_type::<512>::New(512));

    Virtual_file_system
        .Mount_static_device(Task, &Device_path, Device)
        .unwrap();

    let Device_file = File_type::Open(
        &Virtual_file_system,
        Device_path,
        Mode_type::Read_write.into(),
    )
    .await
    .unwrap();

    let Data = 0x1234567890ABCDEF_u64;

    Device_file.Write(&Data.to_le_bytes()).unwrap();

    Device_file.Set_position(&Position_type::Start(0)).unwrap();

    let mut Buffer = [0; 8];

    Device_file.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, Data.to_le_bytes());

    core::mem::drop(Device_file);

    Virtual_file_system.Remove(Device_path).unwrap();
}
