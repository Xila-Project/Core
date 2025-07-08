use core::borrow::Borrow;

use alloc::{borrow::ToOwned, format, string::ToString};

use super::*;

/// A borrowed path type.
/// The implementation is very similar to the standard library's [`std::path::Path`].
/// However, this implementation is more lightweight and allows for std-less usage.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Path_type(str);

impl Path_type {
    pub const ROOT: &'static Path_type = Self::From_str("/");
    pub const EMPTY: &'static Path_type = Self::From_str("");

    /// Contains the OS core, including the kernel, init system, and critical drivers.
    /// Prevents modification by regular users.
    pub const SYSTEM: &'static Path_type = Self::From_str("/System");

    /// Stores system-wide settings in a structured format (e.g., JSON, TOML).
    pub const DEVICES: &'static Path_type = Self::From_str("/Devices");

    /// Hardware devices, symlinks for human-friendly names.
    pub const CONFIGURATION: &'static Path_type = Self::From_str("/Configuration");

    /// Contains the shared configurations between applications.
    pub const SHARED_CONFIGURATION: &'static Path_type = Self::From_str("/Configuration/Shared");

    /// Binaries data.
    pub const DATA: &'static Path_type = Self::From_str("/Data");

    /// Shared data between binaries.
    pub const SHARED_DATA: &'static Path_type = Self::From_str("/Data/Shared");

    /// Contains the system's binaries, including the shell and other executables.
    pub const BINARIES: &'static Path_type = Self::From_str("/Binaries");

    /// Contains the user's data, including documents, downloads, and other files.
    pub const USERS: &'static Path_type = Self::From_str("/Users");

    /// Contains temporary files, including logs and caches.
    pub const TEMPORARY: &'static Path_type = Self::From_str("/Temporary");

    /// Contains logs, including system logs and application logs.
    pub const LOGS: &'static Path_type = Self::From_str("/Temporary/Logs");

    /// # Safety
    /// The caller must ensure that the string is a valid path string.
    pub const fn From_str(Path: &str) -> &Path_type {
        unsafe { &*(Path as *const str as *const Path_type) }
    }

    /// # Safety
    /// The caller must ensure that the string is a valid path string.
    pub fn New<S: AsRef<str> + ?Sized>(Path: &S) -> &Path_type {
        unsafe { &*(Path.as_ref() as *const str as *const Path_type) }
    }

    pub fn is_valid(&self) -> bool {
        is_valid_string(&self.0)
    }

    pub fn is_absolute(&self) -> bool {
        self.0.starts_with('/')
    }

    pub fn is_root(&self) -> bool {
        &self.0 == "/"
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_canonical(&self) -> bool {
        self.0.chars().all(|c| c != '.')
    }

    pub fn Go_parent(&self) -> Option<&Path_type> {
        let characters_to_remove = match self.0.rfind(SEPARATOR) {
            Some(index) => index,
            None => {
                // If there is no separator, the path is either empty or relative to the current directory.
                if self.get_length() > 0 {
                    // Relative to the current directory.
                    return Some(Self::EMPTY);
                } else {
                    return None;
                }
            }
        };

        if characters_to_remove == 0 {
            if self.get_length() == 1 {
                return None;
            }

            if self.is_absolute() {
                return Some(Self::ROOT);
            } else {
                return Some(Self::From_str(""));
            }
        }

        Some(Self::From_str(&self.0[..characters_to_remove]))
    }

    pub fn get_file_prefix(&self) -> Option<&str> {
        let extension_start = self
            .0
            .rfind(EXTENSION_SEPARATOR)
            .or_else(|| Some(self.get_length()))?; // Find the extension separator.
        let file_prefix_start = self.0.rfind(SEPARATOR).map(|i| i + 1).unwrap_or(0); // Find the file prefix start.

        if extension_start <= file_prefix_start {
            return None;
        }

        Some(&self.0[file_prefix_start..extension_start])
    }

    pub fn get_file_name(&self) -> Option<&str> {
        let file_prefix_start = self.0.rfind(SEPARATOR).map(|i| i + 1).unwrap_or(0); // Find the file prefix start.

        if file_prefix_start >= self.get_length() {
            return None;
        }

        Some(&self.0[file_prefix_start..])
    }

    pub fn get_extension(&self) -> Option<&str> {
        let extension_start = self.0.rfind(EXTENSION_SEPARATOR)?;

        Some(&self.0[extension_start + 1..])
    }

    pub fn Set_extension(&self, Extension: &str) -> Option<Path_owned_type> {
        let extension_start = self
            .0
            .rfind(EXTENSION_SEPARATOR)
            .unwrap_or(self.get_length());

        Some(unsafe {
            Path_owned_type::New_unchecked(format!("{}{}", &self.0[..extension_start], Extension))
        })
    }

    pub fn Strip_prefix<'b>(&'b self, Path_prefix: &Path_type) -> Option<&'b Path_type> {
        let mut stripped_prefix = self.0.strip_prefix(&Path_prefix.0)?;

        if stripped_prefix.starts_with(SEPARATOR) {
            stripped_prefix = &stripped_prefix[1..]
        }

        Some(Self::From_str(stripped_prefix))
    }

    pub fn Strip_prefix_absolute<'b>(&'b self, Path_prefix: &Path_type) -> Option<&'b Path_type> {
        if Path_prefix.is_root() {
            return Some(self);
        }

        let Stripped_prefix = self.0.strip_prefix(&Path_prefix.0)?;

        if Stripped_prefix.is_empty() {
            return Some(Self::ROOT);
        }

        Some(Self::From_str(Stripped_prefix))
    }

    pub fn Strip_suffix<'b>(&'b self, Path_suffix: &Path_type) -> Option<&'b Path_type> {
        let stripped_suffix = self.0.strip_suffix(&Path_suffix.0)?;

        if stripped_suffix.ends_with(SEPARATOR) {
            return Some(Self::From_str(
                &stripped_suffix[..stripped_suffix.len() - 1],
            ));
        }

        Some(Self::From_str(stripped_suffix))
    }

    pub fn get_components(&self) -> Components_type {
        Components_type::New(self)
    }

    pub fn Join(&self, Path: &Path_type) -> Option<Path_owned_type> {
        self.to_owned().Join(Path)
    }

    pub fn Append(&self, Path: &str) -> Option<Path_owned_type> {
        self.to_owned().Append(Path)
    }

    pub fn get_length(&self) -> usize {
        self.0.len()
    }

    pub fn As_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "std")]
impl AsRef<std::path::Path> for Path_type {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(&self.0)
    }
}

impl ToOwned for Path_type {
    type Owned = Path_owned_type;

    fn to_owned(&self) -> Self::Owned {
        unsafe { Path_owned_type::New_unchecked(self.0.to_string()) }
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }
}

impl Borrow<Path_type> for Path_owned_type {
    fn borrow(&self) -> &Path_type {
        Path_type::From_str(&self.0)
    }
}

impl AsRef<Path_type> for str {
    fn as_ref(&self) -> &Path_type {
        Path_type::From_str(self)
    }
}

impl AsRef<str> for &Path_type {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path_type> for Path_type {
    fn as_ref(&self) -> &Path_type {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_prefix() {
        let Path = Path_type::From_str("/home/user/file.txt");
        let Prefix = Path_type::From_str("/home/user");
        assert_eq!(Path.Strip_prefix(Prefix).unwrap().As_str(), "file.txt");

        let Path = Path_type::From_str("/home/user/file.txt");
        let Prefix = Path_type::From_str("/");
        assert_eq!(
            Path.Strip_prefix(Prefix).unwrap().As_str(),
            "home/user/file.txt"
        );

        let Invalid_prefix = Path_type::From_str("/home/invalid/");
        assert_eq!(Path.Strip_prefix(Invalid_prefix), None);
    }

    #[test]
    fn test_strip_prefix_absolute() {
        let Path = Path_type::From_str("/");
        let Prefix = Path_type::From_str("/");
        assert_eq!(Path.Strip_prefix_absolute(Prefix).unwrap().As_str(), "/");

        let Path = Path_type::From_str("/Foo/Bar");
        let Prefix = Path_type::From_str("/Foo/Bar");
        assert_eq!(Path.Strip_prefix_absolute(Prefix).unwrap().As_str(), "/");

        let Path = Path_type::From_str("/home/user/file.txt");
        let Prefix = Path_type::From_str("/home/user");
        assert_eq!(
            Path.Strip_prefix_absolute(Prefix).unwrap().As_str(),
            "/file.txt"
        );

        let Path = Path_type::From_str("/home/user/file.txt");
        let Prefix = Path_type::From_str("/");
        assert_eq!(
            Path.Strip_prefix_absolute(Prefix).unwrap().As_str(),
            "/home/user/file.txt"
        );

        let Invalid_prefix = Path_type::From_str("/home/invalid/");
        assert_eq!(Path.Strip_prefix_absolute(Invalid_prefix), None);
    }

    #[test]
    fn test_strip_suffix() {
        let Path = Path_type::From_str("/home/user/file.txt");
        let Suffix = Path_type::From_str("user/file.txt");
        assert_eq!(Path.Strip_suffix(Suffix).unwrap().As_str(), "/home");

        let Invalid_suffix = Path_type::From_str("user/invalid.txt");
        assert_eq!(Path.Strip_suffix(Invalid_suffix), None);
    }

    #[test]
    fn test_go_parent() {
        let Path = Path_type::From_str("/home/user/file.txt");
        assert_eq!(&Path.Go_parent().unwrap().0, "/home/user");
        assert_eq!(&Path.Go_parent().unwrap().Go_parent().unwrap().0, "/home");
        assert_eq!(
            &Path
                .Go_parent()
                .unwrap()
                .Go_parent()
                .unwrap()
                .Go_parent()
                .unwrap()
                .0,
            "/"
        );
        assert_eq!(
            Path.Go_parent()
                .unwrap()
                .Go_parent()
                .unwrap()
                .Go_parent()
                .unwrap()
                .Go_parent(),
            None
        );

        let Path = Path_type::From_str("home/user/file.txt");
        assert_eq!(&Path.Go_parent().unwrap().0, "home/user");
        assert_eq!(&Path.Go_parent().unwrap().Go_parent().unwrap().0, "home");
        assert_eq!(
            &Path
                .Go_parent()
                .unwrap()
                .Go_parent()
                .unwrap()
                .Go_parent()
                .unwrap()
                .0,
            ""
        );
    }

    #[test]
    fn test_path_file() {
        // Regular case
        let Path = Path_type::From_str("/Directory/File.txt");
        assert_eq!(Path.get_extension(), Some("txt"));
        assert_eq!(Path.get_file_prefix(), Some("File"));
        assert_eq!(Path.get_file_name(), Some("File.txt"));

        // No extension
        let Path = Path_type::From_str("/Directory/File");
        assert_eq!(Path.get_extension(), None);
        assert_eq!(Path.get_file_prefix(), Some("File"));
        assert_eq!(Path.get_file_name(), Some("File"));

        // No file prefix
        let Path = Path_type::From_str("File.txt");
        assert_eq!(Path.get_extension(), Some("txt"));
        assert_eq!(Path.get_file_prefix(), Some("File"));
        assert_eq!(Path.get_file_name(), Some("File.txt"));

        // No file prefix or extension
        let Path = Path_type::From_str("/");
        assert_eq!(Path.get_extension(), None);
        assert_eq!(Path.get_file_prefix(), None);
        assert_eq!(Path.get_file_name(), None);

        // No file prefix or extension
        let Path = Path_type::From_str("File");
        assert_eq!(Path.get_extension(), None);
        assert_eq!(Path.get_file_prefix(), Some("File"));
        assert_eq!(Path.get_file_name(), Some("File"));
    }
}
