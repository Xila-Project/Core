



use task::TaskIdentifier;

use file_system::{
    create_device, Create_file_system, Flags_type, Memory_device_type, Mode_type, Open_type,
    Path_type, Position_type, Status_type,
};
#[cfg(target_os = "linux")]
use task::Test;
use virtual_file_system::{FileType, VirtualFileSystemType};

async fn Initialize<'a>() -> (TaskIdentifier, VirtualFileSystemType<'a>) {
    let Task_instance = task::Initialize();

    unsafe {
        let _ = Task_instance.Register_task().await;
    }

    let _ = users::Initialize();

    let _ = time::Initialize(create_device!(drivers::native::Time_driver_type::new()));

    let Task = Task_instance
        .get_current_task_identifier()
        .expect("Failed to get current task identifier");

    let Device = create_device!(Memory_device_type::<512>::new(1024 * 512));

    let Cache_size = 256;

    little_fs::File_system_type::Format(Device.clone(), Cache_size).unwrap();
    let File_system = little_fs::File_system_type::new(Device, Cache_size).unwrap();

    let Virtual_file_system = VirtualFileSystemType::new(
        Task_instance,
        users::get_instance(),
        time::get_instance(),
        Create_file_system!(File_system),
        None,
    ).await
    .unwrap();

    (Task, Virtual_file_system)
}

#[cfg(target_os = "linux")]
#[test]
async fn test_file() {
    let (_, Virtual_file_system) = Initialize();

    let File_path = "/file";

    let File = FileType::Open(
        &Virtual_file_system,
        File_path,
        Flags_type::new(Mode_type::Read_write, Some(Open_type::Create_only), None),
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
#[test]
async fn test_unnamed_pipe() {
    let (Task, Virtual_file_system) = Initialize();

    let (Pipe_read, Pipe_write) =
        FileType::Create_unnamed_pipe(&Virtual_file_system, 512, Status_type::default(), Task)
            .unwrap();

    let Data = b"Hello, world!";

    Pipe_write.Write(Data).unwrap();

    let mut Buffer = [0; 13];

    Pipe_read.Read(&mut Buffer).unwrap();

    assert_eq!(Buffer, *Data);
}

#[cfg(target_os = "linux")]
#[test]
async fn test_named_pipe() {
    let (Task, Virtual_file_system) = Initialize();

    let Pipe_path = "/pipe";

    Virtual_file_system
        .Create_named_pipe(&Pipe_path, 512, Task)
        .unwrap();

    let Pipe_read =
        FileType::Open(&Virtual_file_system, Pipe_path, Mode_type::Read_only.into()).unwrap();

    let Pipe_write = FileType::Open(
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
#[test]
fn test_device() {
    let (Task, Virtual_file_system) = Initialize();

    const Device_path: &Path_type = Path_type::From_str("/Device");

    let Device = create_device!(Memory_device_type::<512>::new(512));

    Virtual_file_system
        .Mount_static_device(Task, &Device_path, Device)
        .unwrap();

    let Device_file = FileType::Open(
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
