use File_system::{
    Entry_type, Error_type, File_identifier_inner_type, File_identifier_type, File_system_traits,
    Flags_type, Path_owned_type, Path_type, Position_type, Result_type, Size_type, Statistics_type,
    Time_type, Type_type, Virtual_file_system_type,
};

use std::collections::BTreeMap;
use std::env::{current_dir, var};
use std::fs::*;
use std::io::{ErrorKind, Read, Seek, Write};

use std::os::unix::fs::DirEntryExt;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use Task::Task_identifier_type;

pub fn Mount_file_systems(Virtual_file_system: &Virtual_file_system_type) -> Result<(), String> {
    let File_system = File_system_type::New().map_err(|Error| format!("{:?}", Error))?;

    Virtual_file_system
        .Mount(Box::new(File_system), Path_type::Get_root())
        .map_err(|Error| format!("{:?}", Error))?;

    Ok(())
}

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
        .write(Flags.Get_mode().Get_write())
        .append(Flags.Get_status().Get_append())
        .create(Flags.Get_open().Get_create())
        .create_new(Flags.Get_open().Get_create_only())
        .truncate(Flags.Get_open().Get_truncate());
}

enum Inner_item_type {
    File(File),
    Directory(ReadDir),
}

type Inner_file_type = Arc<RwLock<(Inner_item_type, Flags_type)>>;

pub struct File_system_type {
    Virtual_root_path: Path_owned_type,
    Open_files: RwLock<BTreeMap<usize, Inner_file_type>>,
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

    fn Get_new_file_identifier<T>(
        Task_identifier: Task_identifier_type,
        Open_files: &BTreeMap<usize, T>,
    ) -> Result_type<File_identifier_type> {
        let Start = Self::Get_local_file_identifier(Task_identifier, File_identifier_type::Minimum);
        let End = Self::Get_local_file_identifier(Task_identifier, File_identifier_type::Maximum);

        // Iterate over the range of file identifiers.
        for i in Start..End {
            if !Open_files.contains_key(&i) {
                return Ok(File_identifier_type::from(i as File_identifier_inner_type));
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

        let mut Open_files = self.Open_files.write()?;

        let File_identifier = Self::Get_new_file_identifier(Task_identifier, &*Open_files)?;

        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        if Flags.Get_open().Get_directory() {
            if Flags.Get_open().Get_create() {
                let Result = create_dir(Full_path.as_ref() as &Path_type);

                if let Err(Error) = Result {
                    if Error.kind() == ErrorKind::AlreadyExists {
                        if Flags.Get_open().Get_create_only() {
                            return Err(Error_type::Already_exists);
                        }
                    } else {
                        return Err(Error.into());
                    }
                }
            }
            let Directory_stream = read_dir(Full_path.as_ref() as &Path_type)?;

            if Open_files
                .insert(
                    Local_file_identifier,
                    Arc::new(RwLock::new((
                        Inner_item_type::Directory(Directory_stream),
                        Flags,
                    ))),
                )
                .is_some()
            {
                return Err(Error_type::Internal_error);
            }

            Ok(File_identifier)
        } else {
            let mut Open_options = OpenOptions::new();

            Apply_flags_to_open_options(Flags, &mut Open_options);

            let File = Open_options
                .open(Full_path.as_ref() as &Path_type)
                .map_err(|Error| Error.kind())?;

            if Open_files
                .insert(
                    Local_file_identifier,
                    Arc::new(RwLock::new((Inner_item_type::File(File), Flags))),
                )
                .is_some()
            {
                return Err(Error_type::Internal_error);
            }

            Ok(File_identifier)
        }
    }

    fn Read(
        &self,
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
        Buffer: &mut [u8],
    ) -> Result_type<Size_type> {
        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        match &mut self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .0
        {
            Inner_item_type::File(File) => Ok(File.read(Buffer)?.into()),

            Inner_item_type::Directory(Directory_stream) => {
                let Directory_entry: &mut Entry_type =
                    Buffer.try_into().map_err(|_| Error_type::Invalid_input)?;

                match Directory_stream.next() {
                    Some(Ok(Entry)) => {
                        Directory_entry.Set_inode(Entry.ino().into());
                        Directory_entry.Set_name(Entry.file_name().to_string_lossy().to_string());
                        Directory_entry.Set_type(From_file_type(Entry.file_type()?));

                        Ok(size_of::<Entry_type>().into())
                    }

                    Some(Err(Error)) => Err(Error.kind().into()),

                    None => Ok(0_usize.into()),
                }
            }
        }
    }

    fn Write(
        &self,
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
        Buffer: &[u8],
    ) -> Result_type<Size_type> {
        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        match &mut self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .0
        {
            Inner_item_type::File(File) => Ok(File.write(Buffer)?.into()),
            _ => Err(Error_type::Unsupported_operation),
        }
    }

    fn Flush(&self, Task: Task_identifier_type, File: File_identifier_type) -> Result_type<()> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        match &mut self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .0
        {
            Inner_item_type::File(File) => File.flush()?,
            _ => return Err(Error_type::Unsupported_operation),
        }

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

    fn Set_position(
        &self,
        Task_identifier: Task_identifier_type,
        File_identifier: File_identifier_type,
        Position_type: &Position_type,
    ) -> Result_type<Size_type> {
        let Local_file_identifier =
            Self::Get_local_file_identifier(Task_identifier, File_identifier);

        match &mut self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .write()?
            .0
        {
            Inner_item_type::File(File) => Ok(File.seek((*Position_type).into())?.into()),
            _ => Err(Error_type::Unsupported_operation),
        }
    }

    fn Delete(&self, Path: &dyn AsRef<Path_type>) -> Result_type<()> {
        let Full_path = self.Get_full_path(&Path)?;

        remove_file(Full_path.as_ref() as &Path_type).map_err(|Error| Error.kind().into())
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
        New_file_identifier: Option<File_identifier_type>,
    ) -> Result_type<File_identifier_type> {
        let Old_local_file_identifier =
            Self::Get_local_file_identifier(Old_task, Old_file_identifier);

        let mut Open_files = self.Open_files.write()?;

        let New_file_identifier = if let Some(New_file_identifier) = New_file_identifier {
            New_file_identifier
        } else {
            Self::Get_new_file_identifier(New_task, &Open_files)?
        };

        let New_local_file_identifier =
            Self::Get_local_file_identifier(New_task, New_file_identifier);

        if Open_files.contains_key(&New_local_file_identifier) {
            return Err(Error_type::Invalid_identifier);
        }

        let File = Open_files
            .remove(&Old_local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?;

        if Open_files.insert(New_local_file_identifier, File).is_some() {
            // Should never happen.
            return Err(Error_type::Internal_error);
        }

        Ok(New_file_identifier)
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

    fn Get_statistics(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
        File_system: File_system::File_system_identifier_type,
    ) -> Result_type<File_system::Statistics_type> {
        use std::os::unix::fs::MetadataExt;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let Open_files = self.Open_files.read()?;

        let Metadata = match &Open_files
            .get(&Local_file_identifier)
            .ok_or(Error_type::Invalid_identifier)?
            .read()?
            .0
        {
            Inner_item_type::File(File) => File.metadata()?,
            Inner_item_type::Directory(_) => return Err(Error_type::Unsupported_operation),
        };

        Ok(Statistics_type::New(
            File_system,
            Metadata.ino().into(),
            Metadata.nlink(),
            Metadata.len().into(),
            Time_type::from(
                Metadata
                    .accessed()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
            ),
            Time_type::from(
                Metadata
                    .modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
            ),
            Time_type::from(
                Metadata
                    .created()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
            ),
            From_file_type(Metadata.file_type()),
        ))
    }

    fn Duplicate_file_identifier(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_identifier_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        let File = self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Not_found)?
            .clone();

        let mut Open_files = self.Open_files.write()?;

        let New_file_identifier = Self::Get_new_file_identifier(Task, &*Open_files)?;

        let Local_file_identifier = Self::Get_local_file_identifier(Task, New_file_identifier);

        if Open_files.insert(Local_file_identifier, File).is_some() {
            // Should never happen.
            return Err(Error_type::Internal_error);
        }

        Ok(New_file_identifier)
    }

    fn Get_mode(
        &self,
        Task: Task_identifier_type,
        File: File_identifier_type,
    ) -> Result_type<File_system::Mode_type> {
        let Local_file_identifier = Self::Get_local_file_identifier(Task, File);

        Ok(self
            .Open_files
            .read()?
            .get(&Local_file_identifier)
            .ok_or(Error_type::Not_found)?
            .read()?
            .1
            .Get_mode())
    }
}

// - Test
#[cfg(test)]
mod Tests {}
