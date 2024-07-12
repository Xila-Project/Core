use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use super::{Extension_separator, Path_type, Separator};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[repr(transparent)]
pub struct Path_owned_type(String);

impl Path_owned_type {
    /// # Safety
    /// The caller must ensure that the string is valid path string.
    pub unsafe fn New_unchecked(Path: String) -> Self {
        Path_owned_type(Path)
    }

    pub fn New(Path: String) -> Option<Self> {
        let Path = if Path.ends_with(Separator) && Path.len() > 1 {
            Path[..Path.len() - 1].to_string()
        } else {
            Path
        };

        if Is_valid_string(&Path) {
            Some(Path_owned_type(Path))
        } else {
            None
        }
    }

    pub fn Root() -> Path_owned_type {
        Path_owned_type("/".to_string())
    }

    pub fn Join(mut self, Path: impl AsRef<Path_type>) -> Option<Self> {
        if Path.as_ref().Is_absolute() {
            return None;
        }

        if Path.as_ref().Is_empty() {
            return Some(self);
        }

        if !self.0.ends_with(Separator) {
            self.0.push(Separator);
        }
        self.0.push_str(Path.as_ref().As_str());

        Some(self)
    }

    pub fn Append(self, Path: &str) -> Option<Self> {
        self.Join(Path_type::New(Path)?)
    }

    pub fn Revert_parent_directory(&mut self) -> &mut Self {
        let mut Last_index = 0;
        for (i, c) in self.0.chars().enumerate() {
            if c == Separator {
                Last_index = i;
            }
        }
        if Last_index == 0 {
            self.0.clear();
            return self;
        }

        self.0.truncate(Last_index);
        self
    }

    pub fn Get_extension(&self) -> Option<&str> {
        let mut extension = None;

        for (i, c) in self.0.chars().enumerate() {
            if c == Extension_separator {
                extension = Some(&self.0[i..]);
            }
        }
        extension
    }

    pub fn Get_file_name(&self) -> &str {
        let mut Last_index = 0;
        for (i, c) in self.0.chars().enumerate() {
            if c == Separator {
                Last_index = i;
            }
        }
        if Last_index >= self.0.len() {
            return &self.0[Last_index..];
        }
        &self.0[Last_index + 1..]
    }

    pub fn Get_relative_to(&self, Path: &Path_owned_type) -> Option<Path_owned_type> {
        if !self.0.starts_with(Path.0.as_str()) {
            return None;
        }

        Some(Path_owned_type(self.0[Path.0.len()..].to_string()))
    }

    pub fn Canonicalize(mut self) -> Self {
        let mut Stack: Vec<&str> = Vec::new();

        if self.Is_absolute() {
            Stack.push("");
        }

        for Component in self.0.split('/') {
            match Component {
                ".." => {
                    Stack.pop();
                }
                "." | "" => continue,
                _ => Stack.push(Component),
            }
        }

        self.0 = Stack.join("/");

        self
    }
}

pub fn Is_valid_string(String: &str) -> bool {
    let Invalid = ['\0', ':', '*', '?', '"', '<', '>', '|', ' '];

    for Character in String.chars() {
        // Check if the string contains invalid characters.
        if Invalid.contains(&Character) {
            return false;
        }
    }

    if String.ends_with(Separator) && String.len() > 1 {
        // Check if the string ends with a separator and is not the root directory.
        return false;
    }

    true
}

impl TryFrom<&str> for Path_owned_type {
    type Error = ();

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        if Is_valid_string(item) {
            Ok(Path_owned_type(item.to_string()))
        } else {
            Err(())
        }
    }
}

impl TryFrom<String> for Path_owned_type {
    type Error = ();

    fn try_from(item: String) -> Result<Self, Self::Error> {
        if Is_valid_string(&item) {
            Ok(Path_owned_type(item))
        } else {
            Err(())
        }
    }
}

impl Display for Path_owned_type {
    fn fmt(&self, Formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(Formatter, "{}", self.0)
    }
}

impl AsRef<str> for Path_owned_type {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for Path_owned_type {
    type Target = Path_type;

    fn deref(&self) -> &Self::Target {
        unsafe { Path_type::New_unchecked(self.0.as_str()) }
    }
}

impl AsRef<Path_type> for Path_owned_type {
    fn as_ref(&self) -> &Path_type {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_path_addition() {
        let Path = Path_owned_type::try_from("/").unwrap();
        assert_eq!(Path.As_str(), "/");
        let Path = Path.Append("Folder").unwrap();
        assert_eq!(Path.As_str(), "/Folder");
        let Path = Path.Append("File").unwrap();
        assert_eq!(Path.As_str(), "/Folder/File");
    }

    #[test]
    fn Test_valid_string() {
        assert!(Is_valid_string("Hello"));
        assert!(Is_valid_string("Hello/World"));
        assert!(Is_valid_string("Hello/World.txt"));
        assert!(!Is_valid_string("Hello/World.txt/"));
        assert!(!Is_valid_string("Hello/World.txt:"));
        assert!(!Is_valid_string("Hello/World.txt*"));
        assert!(!Is_valid_string("Hello/World.txt?"));
        assert!(!Is_valid_string("Hello/World.txt\""));
        assert!(!Is_valid_string("Hello/World.txt<"));
        assert!(!Is_valid_string("Hello/World.txt>"));
        assert!(!Is_valid_string("Hello/World.txt|"));
        assert!(!Is_valid_string("Hello/World.txt "));
        assert!(!Is_valid_string("Hello/World.txt\0"));
        assert!(Is_valid_string(""));
        assert!(!Is_valid_string("Hello/Wo rld.txt/"));
    }

    #[test]
    fn Test_canonicalize() {
        let Path = Path_owned_type::try_from("/home/../home/user/./file.txt").unwrap();
        assert_eq!(Path.Canonicalize().As_str(), "/home/user/file.txt");

        let Path = Path_owned_type::try_from("./home/../home/user/./file.txt").unwrap();
        assert_eq!(Path.Canonicalize().As_str(), "home/user/file.txt");
    }
}
