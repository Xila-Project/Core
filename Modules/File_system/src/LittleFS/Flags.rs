use super::littlefs;
use crate::Flags_type;

pub fn Convert_flags(Flags: Flags_type) -> i32 {
    let mut Little_fs_flags: i32 = if Flags.Get_mode().Get_read() {
        if Flags.Get_mode().Get_write() {
            littlefs::lfs_open_flags_LFS_O_RDWR as i32
        } else {
            littlefs::lfs_open_flags_LFS_O_RDONLY as i32
        }
    } else if Flags.Get_mode().Get_write() {
        littlefs::lfs_open_flags_LFS_O_WRONLY as i32
    } else {
        littlefs::lfs_open_flags_LFS_O_RDONLY as i32
    };

    if Flags.Get_open().Get_create() {
        Little_fs_flags |= littlefs::lfs_open_flags_LFS_O_CREAT as i32;
    }

    if Flags.Get_open().Get_exclusive() {
        Little_fs_flags |= littlefs::lfs_open_flags_LFS_O_EXCL as i32;
    }

    if Flags.Get_open().Get_truncate() {
        Little_fs_flags |= littlefs::lfs_open_flags_LFS_O_TRUNC as i32;
    }

    if Flags.Get_status().Get_append() {
        Little_fs_flags |= littlefs::lfs_open_flags_LFS_O_APPEND as i32;
    }

    Little_fs_flags
}
