use File_system::{
    Error_type, File_identifier_type, File_system_traits, Flags_type, Path_owned_type, Path_type,
    Position_type, Result_type, Size_type, Type_type,
};

use std::collections::BTreeMap;
use std::env::{current_dir, var};
use std::fs::*;
use std::io::{ErrorKind, Read, Seek, Write};

use std::sync::RwLock;

use Task::Task_identifier_type;

fn From_file_type(value: FileType) -> Type_type {
    if value.is_dir() {
        return Type_type::Directory;
    } else if value.is_symlink() {
        return Type_type::Symbolic_link;
    }
    Type_type::File
}

fn Apply_flags_to_open_options(Flags: Flags_type, Open_options: &mut OpenOptions) {
    Open_options
        .read(Flags.Get_mode().Get_read())
        .write(Flags.Get_mode().Get_write() || Flags.Get_status().Get_append());
}

pub struct File_system_type {
    Virtual_root_path: Path_owned_type,
    Open_files: RwLock<BTreeMap<u32, RwLock<File>>>,
}

impl File_system_type {
    pub fn New() -> Result_type<Self> {
        Ok(File_system_type {
            Virtual_root_path: Self::Get_root_path().ok_or(Error_type::Unknown)?,
            Open_files: RwLock::new(BTreeMap::new()),
        })
    }

    fn Get_root_path() -> Option<Path_owned_type> {
        let Root_path = match var("Xila_virtual_root_path") {
            Ok(value) => value,
            Err(_) => match current_dir() {
                Ok(value) => value.to_str()?.to_string(),
                Err(_) => {
                    return None;
                }
            },
        };

        let Root_path = Path_owned_type::try_from(Root_path).ok()?.Append("Xila")?;

        match create_dir(Root_path.as_ref() as &Path_type) {
            Ok(_) => {}
            Err(Error) => {
                if ErrorKind::AlreadyExists != Error.kind() {
                    return None;
                }
            }
        }

        Some(Root_path)
    }

    fn Get_new_file_identifier(
        Task_identifier: Task_identifier_type,
        Open_files: &BTreeMap<u32, RwLock<File>>,
    ) -> Result_type<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task_identifier, File_identifier_type::from(0));
        let End =
            Self::Get_local_file_identifier(Task_identifier, File_identifier_type::from(0xFFFF));

        for i in Start..End {
            if !Open_files.contains_key(&i) {
                return Ok(File_identifier_type::from(i as u16));
            }
        }

        Err(Error_type::Too_many_open_files)
    }

    pub fn Get_full_path(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Path_owned_type> {
        self.Virtual_root_path
            .clone()
            .Join(
                Path.as_ref()
                    .Strip_prefix(Path_type::Get_root())
                    .ok_or(Error_type::Invalid_path)?,
            )
            .ok_or(Error_type::Invalid_path)
    }
}

impl File_system_traits for File_system_type {
    fn Exists(&self, Path: &dyn AsRef<Path_type>) -> Result_type<bool> {
        metadata(self.Get_full_path(&Path)?.as_ref() as &Path_type)
            .map(|_| true)
            .or_else(|Error| match Error.kind() {
                ErrorKind::NotFound => Ok(false),
                _ => Err(Error.kind().into()),
            })
    }

    fn Open(
        &self,
        Task_identifier: Task_identifier_type,
        Path: &dyn AsRef<Path_type>,
        Flags: Flags_type,
    ) -> Result_type<File_identifier_type> {
        let Full_path = self.Get_full_path(&Path)?;

        let mut Open_options = OpenOptions::new();

        Apply_flags_to_open_options(Flags, &mut Open_options);

        let File = Open_options
            .open(Full_path.as_ref() as &Path_type)
            .map_err(|Error| Error.kind())?;

        let mut Open_files = self.Open_files.write()?;

        let File_identifier = Self::Get_new_file_identifier(Task_identifier, &Open_files)?;

        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        if Open_files
            .insert(Local_file_identifier, RwLock::new(File))
            .is_some()
        {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier)
    }

    fn Read(
        &self,
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        Ok(self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .read(Buffer)?
            .into())
    }

    fn Write(
        &self,
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        Ok(self
            .Open_files
            .write()?
            .get_mut(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .write(Buffer)?
            .into())
    }

    fn Flush(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);
        self.Open_files
            .write()?
            .get_mut(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .flush()?;
        Ok(())
    }

    fn Close(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);
        self.Open_files
            .write()?
            .remove(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;
        Ok(())
    }

    fn Get_type(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Type_type> {
        let Full_path = self.Get_full_path(&Path)?;
        let Metadata = metadata(Full_path.as_ref() as &Path_type)?;
        Ok(From_file_type(Metadata.file_type()))
    }

    fn Get_size(&self, Path: &dyn AsRef<Path_type>) -> Result_type<Size_type> {
        let Full_path = self.Get_full_path(&Path)?;
        let Metadata = metadata(Full_path.as_ref() as &Path_type)?;
        Ok(Metadata.len().into())
    }

    fn Set_position(
        &self,
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
        Position_type: &Position_type,
    ) -> Result_type<Size_type> {
        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        Ok(self
            .Open_files
            .write()?
            .get_mut(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .seek((*Position_type).into())
            .map_err(|Error| Error.kind())?
            .into())
    }

    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()> {
        let Full_path = self.Get_full_path(&Path)?;

        remove_file(Full_path.as_ref() as &Path_type).map_err(|Error| Error.kind().into())
    }

    fn Create_directory(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()> {
        let Full_path = self.Get_full_path(&Path)?;

        create_dir(Full_path.as_ref() as &Path_type).map_err(|Error| Error.kind().into())
    }

    fn Create_file(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()> {
        let Full_path = self.Get_full_path(&Path)?;

        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(Full_path.as_ref() as &Path_type)?;

        Ok(())
    }

    fn Close_all(&self, Task_identifier: Task_identifier_type) -> Result_type<()> {
        let Start = Self::Get_local_file_identifier(Task_identifier, File_identifier_type::from(0));
        let End =
            Self::Get_local_file_identifier(Task_identifier, File_identifier_type::from(0xFFFF));

        self.Open_files
            .write()?
            .retain(|File_identifier, _| *File_identifier < Start || *File_identifier > End);

        Ok(())
    }

    fn Transfert_file_identifier(
        &self,
        Old_task: Task_identifier_type,
        New_task: Task_identifier_type,
        Old_file_identifier: File_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let Old_local_file_identifier =
            Self::Get_local_file_identifier(Old_task, Old_file_identifier);

        let mut Open_files = self.Open_files.write()?;

        let New_file_identifier = Self::Get_new_file_identifier(New_task, &Open_files)?;
        let New_local_file_identifier =
            Self::Get_local_file_identifier(New_task, New_file_identifier);

        let File = Open_files
            .remove(&Old_local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if Open_files.insert(New_local_file_identifier, File).is_some() {
            return Err(Error_type::Internal_error);
        }

        Ok(File_identifier_type::from(New_local_file_identifier as u16))
    }

    fn Move(
        &self,
        Source: &dyn AsRef<Path_type>,
        Destination: &dyn AsRef<Path_type>,
    ) -> Result_type<()> {
        let Source = self.Get_full_path(Source)?;
        let Destination = self.Get_full_path(Destination)?;

        rename(
            Source.as_ref() as &Path_type,
            Destination.as_ref() as &Path_type,
        )?;
        Ok(())
    }
}

// - Test
#[cfg(test)]
mod Tests {}
