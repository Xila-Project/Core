use super::littlefs;
use file_system::{AccessFlags, CreateFlags, Flags, StateFlags};

pub fn convert_flags(flags: Flags) -> i32 {
    let mode = flags.get_mode();

    let mut little_fs_flags: i32 = 0;

    if mode.contains(AccessFlags::Read) && mode.contains(AccessFlags::Write) {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_RDWR as i32;
    } else if mode.contains(AccessFlags::Write) {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_RDONLY as i32;
    } else {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_WRONLY as i32;
    }

    let open = flags.get_open();

    if open.contains(CreateFlags::Create) {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_CREAT as i32;
    }

    if open.contains(CreateFlags::Exclusive) {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_EXCL as i32;
    }

    if open.contains(CreateFlags::Truncate) {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_TRUNC as i32;
    }

    if flags.get_status().contains(StateFlags::Append) {
        little_fs_flags |= littlefs::lfs_open_flags_LFS_O_APPEND as i32;
    }

    little_fs_flags
}
