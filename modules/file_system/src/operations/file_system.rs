use crate::{Attributes, Context, DirectoryOperations, FileOperations, Flags, Path, Result};

pub trait FileSystemOperations: FileOperations + DirectoryOperations {
    fn mount(&self) -> Result<()> {
        Ok(())
    }
    fn unmount(&self) -> Result<()> {
        Ok(())
    }

    fn lookup_directory(&self, context: &mut Context, path: &Path) -> Result<()>;

    fn lookup_file(&self, context: &mut Context, path: &Path, flags: Flags) -> Result<()>;

    fn create_directory(&self, path: &Path) -> Result<()>;

    fn create_file(&self, path: &Path) -> Result<()>;

    /// Remove a file or directory from the file system.
    ///
    /// Permanently deletes the specified file or directory. For directories,
    /// they must be empty before they can be removed.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    /// * `path` - Path to the file or directory to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File or directory successfully removed
    /// * `Err(Error)` - Error during removal
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`] - File or directory doesn't exist
    /// * [`Error::PermissionDenied`] - Insufficient permissions
    /// * [`Error::DirectoryNotEmpty`] - Directory contains files
    /// * [`Error::RessourceBusy`] - File is currently in use
    fn remove(&self, path: &Path) -> Result<()>;

    /// Rename or move a file or directory.
    ///
    /// Changes the name or location of a file or directory. This can be used
    /// for both renaming within the same directory and moving between directories.
    ///
    /// # Arguments
    ///
    /// * `context` - File system context
    /// * `source` - Current path of the file or directory
    /// * `destination` - New path for the file or directory
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File or directory successfully renamed/moved
    /// * `Err(Error)` - Error during rename operation
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`] - Source file doesn't exist
    /// * [`Error::AlreadyExists`] - Destination already exists
    /// * [`Error::PermissionDenied`] - Insufficient permissions
    fn rename(&self, source: &Path, destination: &Path) -> Result<()>;

    // - Directory

    // - Statistics

    fn get_attributes(&self, path: &Path, attributes: &mut Attributes) -> Result<()>;

    fn set_attributes(&self, path: &Path, attributes: &Attributes) -> Result<()>;
}
