use super::littlefs;
use file_system::Flags;

pub fn convert_flags(flags: Flags) -> i32 {
    let mut little_fs_flags: i32 = if flags.get_mode().get_read() {
        if flags.get_mode().get_write() {
            littlefs::lfs_open_flags_LFS_O_RDWR as i32
        } else {
            littlefs::lfs_open_flags_LFS_O_RDONLY as i32
        }
    } else if flags.get_mode().get_write() {
        littlefs::lfs_open_flags_LFS_O_WRONLY as i32
    } else {
        littlefs::lfs_open_flags_LFS_O_RDONLY as i32
    };

    if flags.get_open().get_create() {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_CREAT as i32;
    }

    if flags.get_open().get_exclusive() {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_EXCL as i32;
    }

    if flags.get_open().get_truncate() {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_TRUNC as i32;
    }

    if flags.get_status().get_append() {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_APPEND as i32;
    }

    little_fs_flags
}
