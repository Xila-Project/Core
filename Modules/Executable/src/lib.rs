#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod Error;
mod Read_data;
mod Standard;

pub use Error::*;
pub use Read_data::*;
pub use Standard::*;

use Task::Join_handle_type;
use Virtual_file_system::File_type;

use File_system::Path_type;

pub fn Execute<P: AsRef<Path_type>>(
    Path: P,
    Inputs: String,
    Standard: Standard_type,
) -> Result_type<Join_handle_type<isize>> {
    let Task_instance = Task::Get_instance();

    let Task = Task_instance.Get_current_task_identifier()?;

    let File = File_type::Open(
        Virtual_file_system::Get_instance(),
        &Path,
        File_system::Mode_type::Read_write.into(),
        Task,
    )?;

    let File_name = Path
        .as_ref()
        .Get_file_name()
        .ok_or(File_system::Error_type::Invalid_path)?;

    let mut Read_data = Read_data_type::New_default();
    File.Read(&mut Read_data)?;
    let Read_data: Read_data_type = Read_data.try_into().unwrap();

    let Main = Read_data
        .Get_main()
        .ok_or(Error_type::Failed_to_get_main_function)?;
    let Stack_size = Read_data.Get_stack_size();

    let (_, Join_handle) =
        Task_instance.New_task(Task, None, File_name, Some(Stack_size), move || {
            let Task = Task::Get_instance().Get_current_task_identifier().unwrap();

            let Standard = Standard.Transfert(Task).unwrap();

            match Main(Standard, Inputs) {
                Ok(_) => 0_isize,
                Err(Error) => -(Error.get() as isize),
            }
        })?;

    Ok(Join_handle)
}
