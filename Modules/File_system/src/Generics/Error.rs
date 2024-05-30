#[derive(Debug, PartialEq)]
pub enum Error_type {
    Failed_to_initialize_file_system,
    Permission_denied,
    File_not_found,
    Directory_not_found,
    File_already_exists,
    Directory_already_exists,
    File_system_full,
    File_system_error,
    Invalid_path,
    Invalid_file,
    Invalid_directory,
    Invalid_symbolic_link,
    Unknown,
    Invalid_file_identifier,
}
