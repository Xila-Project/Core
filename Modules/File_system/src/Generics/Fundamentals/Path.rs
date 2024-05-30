use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign},
};

#[derive(Clone)]
pub struct Path_type(String);

impl From<&str> for Path_type {
    fn from(item: &str) -> Self {
        item.to_string().into()
    }
}

impl From<String> for Path_type {
    fn from(item: String) -> Self {
        Path_type(item)
    }
}

impl Display for Path_type {
    fn fmt(&self, Formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(Formatter, "{}", self.0)
    }
}

impl Add<Path_type> for Path_type {
    type Output = Path_type;

    fn add(mut self, rhs: Path_type) -> Self::Output {
        self.Append(rhs.0.as_str());
        self
    }
}

impl Add<&Path_type> for Path_type {
    type Output = Path_type;

    fn add(mut self, rhs: &Path_type) -> Self::Output {
        self.Append(rhs.0.as_str());
        self
    }
}

impl AddAssign<Path_type> for Path_type {
    fn add_assign(&mut self, rhs: Path_type) {
        self.Append(rhs.0.as_str());
    }
}

impl AddAssign<&Path_type> for Path_type {
    fn add_assign(&mut self, rhs: &Path_type) {
        self.Append(rhs.0.as_str());
    }
}

impl Path_type {
    const Separator: char = '/';
    const Extension_separator: char = '.';

    pub fn New() -> Path_type {
        Path_type("".to_string())
    }

    pub fn Root() -> Path_type {
        Path_type("/".to_string())
    }

    pub fn To_str(&self) -> &str {
        &self.0
    }

    pub fn Append(&mut self, File: &str) -> &mut Self {
        if !self.0.ends_with(Self::Separator) {
            self.0.push(Self::Separator);
        }
        self.0.push_str(File);
        self
    }

    pub fn Revert_parent_directory(&mut self) -> &mut Self {
        let mut Last_index = 0;
        for (i, c) in self.0.chars().enumerate() {
            if c == Self::Separator {
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
            if c == Self::Extension_separator {
                extension = Some(&self.0[i..]);
            }
        }
        extension
    }

    pub fn Get_file_name(&self) -> &str {
        let mut Last_index = 0;
        for (i, c) in self.0.chars().enumerate() {
            if c == Self::Separator {
                Last_index = i;
            }
        }
        if Last_index >= self.0.len() {
            return &self.0[Last_index..];
        }
        &self.0[Last_index + 1..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_path_file_and_directory() {
        let mut Path = Path_type::Root();
        assert_eq!(Path.To_str(), "/");
        Path.Append("Directory");
        assert_eq!(Path.To_str(), "/Directory");
        Path.Append("File");
        assert_eq!(Path.to_string(), "/Directory/File");
        Path.Revert_parent_directory();
        assert_eq!(Path.to_string(), "/Directory");
    }

    #[test]
    fn Test_path_extension() {
        let Path = Path_type::Root();
        assert_eq!(Path.Get_extension(), None);
        let Path = Path_type::from("File");
        assert_eq!(Path.Get_extension(), None);
        let Path = Path_type::from("File.txt");
        assert_eq!(Path.Get_extension(), Some(".txt"));
        let Path = Path_type::from("/Directory/File.txt");
        assert_eq!(Path.Get_extension(), Some(".txt"));
        let Path = Path_type::from("/Directory/File");
        assert_eq!(Path.Get_extension(), None);
    }

    #[test]
    fn Test_path_file_name() {
        let Path = Path_type::from("");
        assert_eq!(Path.Get_file_name(), "");
        let Path = Path_type::from("/File");
        assert_eq!(Path.Get_file_name(), "File");
        let Path = Path_type::from("/File.txt");
        assert_eq!(Path.Get_file_name(), "File.txt");
        let Path = Path_type::from("/Directory/File.txt");
        assert_eq!(Path.Get_file_name(), "File.txt");
        let Path = Path_type::from("/Directory/File");
        assert_eq!(Path.Get_file_name(), "File");
        let Path = Path_type::from("/Directory");
        assert_eq!(Path.Get_file_name(), "Directory");
    }

    #[test]
    fn Test_path_addition() {
        let mut Path = Path_type::from("/");
        Path += Path_type::from("Folder");
        assert_eq!(Path.To_str(), "/Folder");
        Path += Path_type::from("File");
        assert_eq!(Path.To_str(), "/Folder/File");
    }
}
