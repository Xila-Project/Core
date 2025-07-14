extern crate alloc;

use task::TaskIdentifier;

use file_system::{
    create_device, create_file_system, Flags, MemoryDevice, Mode, Open, Path, Position, Status,
};
#[cfg(target_os = "linux")]
use task::test;
use virtual_file_system::{File, VirtualFileSystem};

async fn initialize<'a>() -> (TaskIdentifier, VirtualFileSystem<'a>) {
    let task_instance = task::initialize();

    let _ = users::initialize();

    let _ = time::initialize(create_device!(drivers::native::TimeDriver::new()));

    let task = task_instance.get_current_task_identifier().await;

    let device = create_device!(MemoryDevice::<512>::new(1024 * 512));

    let cache_size = 256;

    little_fs::FileSystem::format(device.clone(), cache_size).unwrap();
    let file_system = little_fs::FileSystem::new(device, cache_size).unwrap();

    let virtual_file_system = VirtualFileSystem::new(
        task_instance,
        users::get_instance(),
        time::get_instance(),
        create_file_system!(file_system),
        None,
    )
    .unwrap();

    (task, virtual_file_system)
}

#[cfg(target_os = "linux")]
#[test]
async fn test_file() {
    let (_, virtual_file_system) = initialize().await;

    let file_path = "/file";

    let file = File::open(
        &virtual_file_system,
        file_path,
        Flags::new(Mode::READ_WRITE, Some(Open::CREATE_ONLY), None),
    )
    .await
    .unwrap();

    let data = b"Hello, world!";

    file.write(data).await.unwrap();

    file.set_position(&Position::Start(0)).await.unwrap();

    let mut buffer = [0; 13];

    file.read(&mut buffer).await.unwrap();

    assert_eq!(buffer, *data);

    core::mem::drop(file);

    let _ = virtual_file_system.remove(file_path).await.unwrap();
}

#[cfg(target_os = "linux")]
#[test]
async fn test_unnamed_pipe() {
    let (task, virtual_file_system) = initialize().await;

    let (pipe_read, pipe_write) =
        File::create_unnamed_pipe(&virtual_file_system, 512, Status::default(), task)
            .await
            .unwrap();

    let data = b"Hello, world!";

    pipe_write.write(data).await.unwrap();

    let mut buffer = [0; 13];

    pipe_read.read(&mut buffer).await.unwrap();

    assert_eq!(buffer, *data);
}

#[cfg(target_os = "linux")]
#[test]
async fn test_named_pipe() {
    let (task, virtual_file_system) = initialize().await;

    let pipe_path = "/pipe";

    virtual_file_system
        .create_named_pipe(&pipe_path, 512, task)
        .await
        .unwrap();

    let pipe_read = File::open(&virtual_file_system, pipe_path, Mode::READ_ONLY.into())
        .await
        .unwrap();

    let pipe_write = File::open(&virtual_file_system, pipe_path, Mode::WRITE_ONLY.into())
        .await
        .unwrap();

    let data = b"Hello, world!";

    pipe_write.write(data).await.unwrap();

    let mut buffer = [0; 13];
    pipe_read.read(&mut buffer).await.unwrap();

    assert_eq!(buffer, *data);

    core::mem::drop(pipe_read);
    core::mem::drop(pipe_write);

    let _ = virtual_file_system.remove(pipe_path).await.unwrap();
}

#[cfg(target_os = "linux")]
#[test]
async fn test_device() {
    let (task, virtual_file_system) = initialize().await;

    const DEVICE_PATH: &Path = Path::from_str("/devices");

    let device = create_device!(MemoryDevice::<512>::new(512));

    virtual_file_system
        .mount_static_device(task, &DEVICE_PATH, device)
        .await
        .unwrap();

    let device_file = File::open(&virtual_file_system, DEVICE_PATH, Mode::READ_WRITE.into())
        .await
        .unwrap();

    let data = 0x1234567890ABCDEF_u64;

    device_file.write(&data.to_le_bytes()).await.unwrap();

    device_file.set_position(&Position::Start(0)).await.unwrap();

    let mut buffer = [0; 8];

    device_file.read(&mut buffer).await.unwrap();

    assert_eq!(buffer, data.to_le_bytes());

    core::mem::drop(device_file);

    let _ = virtual_file_system.remove(DEVICE_PATH).await.unwrap();
}
