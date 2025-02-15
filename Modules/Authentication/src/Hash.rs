use File_system::Mode_type;
use Virtual_file_system::{File_type, Virtual_file_system_type};

use crate::{Error_type, Random_device_path, Result_type};

pub fn Generate_salt<'a>(
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
) -> Result_type<String> {
    let Random_file = File_type::Open(
        Virtual_file_system,
        Random_device_path,
        Mode_type::Read_only.into(),
    )
    .map_err(Error_type::Failed_to_open_random_device)?;

    let mut Buffer = [0_u8; 16];

    Random_file
        .Read(&mut Buffer)
        .map_err(Error_type::Failed_to_read_random_device)?;

    Buffer.iter_mut().for_each(|Byte| {
        *Byte = *Byte % 26 + 97;
    });

    Ok(core::str::from_utf8(&Buffer).unwrap().to_string())
}

pub fn Hash_password(Password: &str, Salt: &str) -> String {
    use sha2::Digest;

    let mut Hasher = sha2::Sha512::new();

    Hasher.update(Password.as_bytes());
    Hasher.update(Salt.as_bytes());

    let Hash = Hasher.finalize();

    format!("{:x}", Hash)
}
