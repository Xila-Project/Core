use std::borrow::Borrow;

use super::*;

/// A borrowed path type.
/// The implementation is very similar to the standard library's `std::path::Path`.
/// However, this implementation is more lightweight and allows for std-less usage.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Path_type(str);

impl Path_type {
    pub fn Get_root() -> &'static Path_type {
        unsafe { Self::New_unchecked("/") }
    }

    pub fn Get_empty() -> &'static Path_type {
        unsafe { Self::New_unchecked("") }
    }

    /// # Safety
    /// The caller must ensure that the string is a valid path string.
    pub const unsafe fn New_unchecked_constant(Path: &'static str) -> &'static Path_type {
        &*(Path as *const str as *const Path_type)
    }

    /// # Safety
    /// The caller must ensure that the string is a valid path string.
    pub unsafe fn New_unchecked<S: AsRef<str> + ?Sized>(Path: &S) -> &Path_type {
        unsafe { &*(Path.as_ref() as *const str as *const Path_type) }
    }

    pub fn New<S: AsRef<str> + ?Sized>(Path: &S) -> Option<&Path_type> {
        let Path = if Path.as_ref().ends_with(Separator) && Path.as_ref().len() > 1 {
            &Path.as_ref()[..Path.as_ref().len() - 1]
        } else {
            Path.as_ref()
        };

        if Is_valid_string(Path) {
            Some(unsafe { Self::New_unchecked(Path) })
        } else {
            None
        }
    }

    pub fn Is_absolute(&self) -> bool {
        self.0.starts_with('/')
    }

    pub fn Is_root(&self) -> bool {
        &self.0 == "/"
    }

    pub fn Is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn Is_canonical(&self) -> bool {
        self.0.chars().all(|c| c != '.')
    }

    pub fn Go_parent(&self) -> Option<&Path_type> {
        let Characters_to_remove = match self.0.rfind(Separator) {
            Some(index) => index,
            None => {
                // If there is no separator, the path is either empty or relative to the current directory.
                if self.Get_length() > 0 {
                    // Relative to the current directory.
                    return Some(Self::Get_empty());
                } else {
                    return None;
                }
            }
        };

        if Characters_to_remove == 0 {
            if self.Get_length() == 1 {
                return None;
            }

            if self.Is_absolute() {
                return Some(Self::Get_root());
            } else {
                return Some(unsafe { Self::New_unchecked("") });
            }
        }

        Some(unsafe { Self::New_unchecked(&self.0[..Characters_to_remove]) })
    }

    pub fn Get_file_prefix(&self) -> Option<&str> {
        let Extension_start = self
            .0
            .rfind(Extension_separator)
            .or_else(|| Some(self.Get_length()))?; // Find the extension separator.
        let File_prefix_start = self.0.rfind(Separator).map(|i| i + 1).unwrap_or(0); // Find the file prefix start.

        if Extension_start <= File_prefix_start {
            return None;
        }

        Some(&self.0[File_prefix_start..Extension_start])
    }

    pub fn Get_file_name(&self) -> Option<&str> {
        let File_prefix_start = self.0.rfind(Separator).map(|i| i + 1).unwrap_or(0); // Find the file prefix start.

        if File_prefix_start >= self.Get_length() {
            return None;
        }

        Some(&self.0[File_prefix_start..])
    }

    pub fn Get_extension(&self) -> Option<&str> {
        let Extension_start = self.0.rfind(Extension_separator)?;

        Some(&self.0[Extension_start..])
    }

    pub fn Set_extension(&self, Extension: &str) -> Option<Path_owned_type> {
        let Extension_start = self
            .0
            .rfind(Extension_separator)
            .unwrap_or(self.Get_length());

        Some(unsafe {
            Path_owned_type::New_unchecked(format!("{}{}", &self.0[..Extension_start], Extension))
        })
    }

    pub fn Strip_prefix<'b>(&'b self, Path_prefix: &Path_type) -> Option<&'b Path_type> {
        let mut Stripped_prefix = self.0.strip_prefix(&Path_prefix.0)?;

        if Stripped_prefix.starts_with(Separator) {
            Stripped_prefix = &Stripped_prefix[1..]
        }

        Self::New(Stripped_prefix)
    }

    pub fn Strip_prefix_absolute<'b>(&'b self, Path_prefix: &Path_type) -> Option<&'b Path_type> {
        if Path_prefix.Is_root() {
            return Some(self);
        }

        let Stripped_prefix = self.0.strip_prefix(&Path_prefix.0)?;

        Self::New(Stripped_prefix)
    }

    pub fn Strip_suffix<'b>(&'b self, Path_suffix: &Path_type) -> Option<&'b Path_type> {
        Self::New(self.0.strip_suffix(&Path_suffix.0)?)
    }

    pub fn Get_components(&self) -> Components_type {
        Components_type::New(self)
    }

    pub fn Join(&self, Path: &Path_type) -> Option<Path_owned_type> {
        self.to_owned().Join(Path)
    }

    pub fn Append(&self, Path: &str) -> Option<Path_owned_type> {
        self.to_owned().Append(Path)
    }

    pub fn Get_length(&self) -> usize {
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
        unsafe { Path_type::New_unchecked(&self.0) }
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
mod Tests {
    use super::*;

    #[test]
    fn Test_strip_prefix() {
        let Path = Path_type::New("/home/user/file.txt").unwrap();
        let Prefix = Path_type::New("/home/user").unwrap();
        assert_eq!(Path.Strip_prefix(Prefix).unwrap().As_str(), "file.txt");

        let Path = Path_type::New("/home/user/file.txt").unwrap();
        let Prefix = Path_type::New("/").unwrap();
        assert_eq!(
            Path.Strip_prefix(Prefix).unwrap().As_str(),
            "home/user/file.txt"
        );

        let Invalid_prefix = Path_type::New("/home/invalid/").unwrap();
        assert_eq!(Path.Strip_prefix(Invalid_prefix), None);
    }

    #[test]
    fn Test_strip_prefix_absolute() {
        let Path = Path_type::New("/home/user/file.txt").unwrap();
        let Prefix = Path_type::New("/home/user").unwrap();
        assert_eq!(
            Path.Strip_prefix_absolute(Prefix).unwrap().As_str(),
            "/file.txt"
        );

        let Path = Path_type::New("/home/user/file.txt").unwrap();
        let Prefix = Path_type::New("/").unwrap();
        assert_eq!(
            Path.Strip_prefix_absolute(Prefix).unwrap().As_str(),
            "/home/user/file.txt"
        );

        let Invalid_prefix = Path_type::New("/home/invalid/").unwrap();
        assert_eq!(Path.Strip_prefix_absolute(Invalid_prefix), None);
    }

    #[test]
    fn Test_strip_suffix() {
        let Path = Path_type::New("/home/user/file.txt").unwrap();
        let Suffix = Path_type::New("user/file.txt").unwrap();
        assert_eq!(Path.Strip_suffix(Suffix).unwrap().As_str(), "/home");

        let Invalid_suffix = Path_type::New("user/invalid.txt").unwrap();
        assert_eq!(Path.Strip_suffix(Invalid_suffix), None);
    }

    #[test]
    fn Test_go_parent() {
        let Path = Path_type::New("/home/user/file.txt").unwrap();
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

        let Path = Path_type::New("home/user/file.txt").unwrap();
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
    fn Test_path_file() {
        // Regular case
        let Path = Path_type::New("/Directory/File.txt").unwrap();
        assert_eq!(Path.Get_extension(), Some(".txt"));
        assert_eq!(Path.Get_file_prefix(), Some("File"));
        assert_eq!(Path.Get_file_name(), Some("File.txt"));

        // No extension
        let Path = Path_type::New("/Directory/File").unwrap();
        assert_eq!(Path.Get_extension(), None);
        assert_eq!(Path.Get_file_prefix(), Some("File"));
        assert_eq!(Path.Get_file_name(), Some("File"));

        // No file prefix
        let Path = Path_type::New("File.txt").unwrap();
        assert_eq!(Path.Get_extension(), Some(".txt"));
        assert_eq!(Path.Get_file_prefix(), Some("File"));
        assert_eq!(Path.Get_file_name(), Some("File.txt"));

        // No file prefix or extension
        let Path = Path_type::New("/").unwrap();
        assert_eq!(Path.Get_extension(), None);
        assert_eq!(Path.Get_file_prefix(), None);
        assert_eq!(Path.Get_file_name(), None);

        // No file prefix or extension
        let Path = Path_type::New("File").unwrap();
        assert_eq!(Path.Get_extension(), None);
        assert_eq!(Path.Get_file_prefix(), Some("File"));
        assert_eq!(Path.Get_file_name(), Some("File"));
    }
}
