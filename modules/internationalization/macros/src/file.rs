use std::{
    fs::{self, DirEntry},
    io::Result,
};

pub fn filter_files(entry: Result<DirEntry>) -> Option<String> {
    let path = match entry {
        Ok(e) => e.path(),
        Err(err) => {
            eprintln!("Failed to read directory entry: {}", err);
            return None;
        }
    };

    if path.extension().and_then(|s| s.to_str()) != Some("po") {
        return None;
    }

    match fs::read_to_string(&path) {
        Ok(content) => Some(content),
        Err(err) => {
            eprintln!("Failed to open PO file {:?}: {}", path, err);
            None
        }
    }
}
