use std::ops::Add;

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

impl ToString for Path_type {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Add<&str> for Path_type {
    type Output = Path_type;

    fn add(mut self, rhs: &str) -> Self::Output {
        self.0.push_str(rhs);
        self
    }
}

impl Add<String> for Path_type {
    type Output = Path_type;

    fn add(mut self, rhs: String) -> Self::Output {
        self.0.push_str(rhs.as_str());
        self
    }
}

impl Add<Path_type> for Path_type {
    type Output = Path_type;

    fn add(mut self, rhs: Path_type) -> Self::Output {
        self.0.push_str(rhs.0.as_str());
        self
    }
}

impl Add<&Path_type> for Path_type {
    type Output = Path_type;

    fn add(mut self, rhs: &Path_type) -> Self::Output {
        self.0.push_str(rhs.0.as_str());
        self
    }
}

impl Path_type {
    const Path_seprarator: char = '/';
    const Extension_separator: char = '.';

    pub fn New() -> Path_type {
        Path_type("".to_string())
    }

    pub fn To_str(&self) -> &str {
        &self.0
    }

    pub fn Add_directory(&mut self, Directory: &str) -> &mut Self {
        self.0.push(Self::Path_seprarator);
        self.0.push_str(Directory);
        self
    }

    pub fn Add_file(&mut self, File: &str) -> &mut Self {
        self.0.push(Self::Path_seprarator);
        self.0.push_str(File);
        self
    }

    pub fn Revert_parent_directory(&mut self) -> &mut Self {
        let mut Index = 0;
        let mut Last_index = 0;
        for c in self.0.chars() {
            if c == Self::Path_seprarator {
                Last_index = Index;
            }
            Index += 1;
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
        let mut index = 0;
        for c in self.0.chars() {
            if c == Self::Extension_separator {
                extension = Some(&self.0[index..]);
            }
            index += 1;
        }
        extension
    }

    pub fn Get_file_name(&self) -> &str {
        let mut Index = 0;
        let mut Last_index = 0;
        for c in self.0.chars() {
            if c == Self::Path_seprarator {
                Last_index = Index;
            }
            Index += 1;
        }
        if Last_index == 0 {
            return &self.0;
        }

        &self.0[Last_index + 1..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Test_path_file_and_directory() {
        let mut Path = Path_type::New();
        let Path = Path.Add_directory("Directory");
        assert_eq!(Path.to_string(), "/Directory");

        let mut Path = Path_type::New();
        let Path = Path.Add_file("File");
        assert_eq!(Path.to_string(), "/File");

        let mut Path = Path_type::New();
        let Path = Path.Add_directory("Directory").Add_file("File");
        assert_eq!(Path.to_string(), "/Directory/File");
    }

    #[test]
    fn Test_path_extension() {
        let Path = Path_type::from("");
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
        let Path = Path_type::from("File");
        assert_eq!(Path.Get_file_name(), "File");
        let Path = Path_type::from("File.txt");
        assert_eq!(Path.Get_file_name(), "File.txt");
        let Path = Path_type::from("/Directory/File.txt");
        assert_eq!(Path.Get_file_name(), "File.txt");
        let Path = Path_type::from("/Directory/File");
        assert_eq!(Path.Get_file_name(), "File");
    }
}
