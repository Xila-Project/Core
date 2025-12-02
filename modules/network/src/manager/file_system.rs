use core::mem::transmute;

use crate::{DirectoryContext, InterfaceContext, Manager, manager::utilities::get_attributes};
use alloc::boxed::Box;
use embassy_net::Stack;
use file_system::{
    Attributes, Context, DirectCharacterDevice, Error, FileSystemOperations, Flags, Kind,
    MountOperations, Path, Permissions, Result,
};
use users::{GroupIdentifier, UserIdentifier};

impl FileSystemOperations for Manager<'_> {
    fn lookup_directory(&self, context: &mut Context, path: &Path) -> Result<()> {
        if !path.is_root() {
            return Err(Error::NotFound);
        }

        let directory_context = DirectoryContext::new();
        context.set_private_data(Box::new(directory_context));

        Ok(())
    }

    fn lookup_file(&self, context: &mut Context, path: &Path, _: Flags) -> Result<()> {
        if path.is_root() {
            return Err(Error::NotFound);
        }

        let interfaces = self
            .interfaces
            .try_read()
            .map_err(|_| Error::RessourceBusy)?;

        let file_name = path.get_file_name().ok_or(Error::NotFound)?;

        let interface = Self::find_interface(&interfaces, file_name).ok_or(Error::NotFound)?;

        let interface = &interfaces[interface];

        let stack = unsafe { transmute::<Stack<'_>, Stack<'static>>(interface.stack.clone()) };
        let controller = unsafe {
            transmute::<&dyn DirectCharacterDevice, &'static dyn DirectCharacterDevice>(
                interface.controller,
            )
        };

        let interface_context = InterfaceContext { stack, controller };

        context.set_private_data(Box::new(interface_context));

        Ok(())
    }

    fn create_directory(&self, _: &Path) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn create_file(&self, _: &Path) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn remove(&self, _: &Path) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn rename(&self, _: &Path, _: &Path) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn get_attributes(&self, path: &Path, attributes: &mut Attributes) -> Result<()> {
        let interfaces = self
            .interfaces
            .try_read()
            .map_err(|_| Error::RessourceBusy)?;

        let file_name = path.get_file_name().ok_or(Error::NotFound)?;

        let interface = Self::find_interface(&interfaces, file_name).ok_or(Error::NotFound)?;

        get_attributes(interface, attributes);

        Ok(())
    }

    fn set_attributes(&self, _: &Path, _: &Attributes) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }
}

impl MountOperations for Manager<'_> {}
