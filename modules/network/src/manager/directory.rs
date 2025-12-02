use crate::Manager;
use file_system::{Context, DirectoryOperations, Entry, Error, Kind, Result, Size};

pub(crate) struct DirectoryContext {
    index: usize,
}

impl DirectoryContext {
    pub fn new() -> Self {
        DirectoryContext { index: 0 }
    }
}

impl DirectoryOperations for Manager<'_> {
    fn read(&self, context: &mut Context) -> Result<Option<Entry>> {
        let context = context
            .get_private_data_mutable_of_type::<DirectoryContext>()
            .ok_or(Error::InvalidContext)?;

        let interfaces = self
            .interfaces
            .try_read()
            .map_err(|_| Error::RessourceBusy)?;

        let interface = match interfaces.get(context.index) {
            Some(interface) => interface,
            None => return Ok(None),
        };

        context.index += 1;

        Ok(Some(Entry {
            inode: context.index as u64,
            name: interface.name.clone().into(),
            kind: Kind::CharacterDevice,
            size: 0,
        }))
    }

    fn set_position(&self, context: &mut Context, position: Size) -> Result<()> {
        let context = context
            .get_private_data_mutable_of_type::<DirectoryContext>()
            .ok_or(Error::InvalidContext)?;

        context.index = position as usize;

        Ok(())
    }

    fn get_position(&self, context: &mut Context) -> Result<Size> {
        let context = context
            .get_private_data_mutable_of_type::<DirectoryContext>()
            .ok_or(Error::InvalidContext)?;

        Ok(context.index as Size)
    }

    fn rewind(&self, context: &mut Context) -> Result<()> {
        let context = context
            .get_private_data_mutable_of_type::<DirectoryContext>()
            .ok_or(Error::InvalidContext)?;

        context.index = 0;

        Ok(())
    }

    fn close(&self, context: &mut Context) -> Result<()> {
        context
            .take_private_data_of_type::<DirectoryContext>()
            .ok_or(Error::InvalidContext)?;
        Ok(())
    }
}
