#![allow(non_snake_case)]

#[cfg(target_os = "linux")]
#[test]
fn Virtual_file_system_test() {
    use File_system::{
        Drivers::Native::File_system_type,
        Prelude::{
            File_type, Mode_type, Path_type, Position_type, Status_type, Virtual_file_system_type,
        },
    };

    let Task_manager = Task::Manager_type::New();

    let Users_manager = Users::Manager_type::New();

    let Virtual_file_system = Virtual_file_system_type::New(Task_manager, Users_manager);

    Virtual_file_system
        .Mount(
            Box::new(File_system_type::New().expect("Failed to create file system")),
            Path_type::Get_root(),
        )
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
        &Virtual_file_system,
        File_path,
        Mode_type::Read_write().into(),
    )
    .expect("Failed to open file");

    let Data = b"Hello, world!";

    File.Write(Data).expect("Failed to write data");

    File.Set_position(&Position_type::Start(0_u64.into()))
        .expect("Failed to set position");

    let mut Buffer = [0; 13];

    File.Read(&mut Buffer).expect("Failed to read data");

    assert_eq!(Buffer, *Data);

    std::mem::drop(File);

    Virtual_file_system
        .Delete(File_path, false)
        .expect("Failed to delete file");

    let (Pipe_read, Pipe_write) =
        File_type::Create_unamed_pipe(&Virtual_file_system, 512, Status_type::default())
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

    /*
    Virtual_file_system
        .Create_named_pipe(&Pipe_path, 512)
        .expect("Failed to create pipe");

    let Pipe_read = File_type::Open(
        &Virtual_file_system,
        Pipe_path,
        Mode_type::Read_only().into(),
    ).expect("Failed to open pipe");

    let mut Buffer = [0; 13];

    Pipe_write.Write(Data).expect("Failed to write data");

    Pipe_read.Read(&mut Buffer).expect("Failed to read data");

    assert_eq!(Buffer, *Data);

    std::mem::drop(Pipe_read);
    std::mem::drop(Pipe_write);

    Virtual_file_system
        .Delete(Pipe_path, false)
        .expect("Failed to delete pipe");
    */
}
