#include "Xila.h"

__wasi_errno_t Into_Wasi_Error(File_system_result_type Error)
{
    switch (Error)
    {
    case 0:
        return __WASI_ESUCCESS;
    case Not_found:
        return __WASI_ENOENT;
    default:
        return __WASI_ECANCELED;
        break;
    }
}

void Into_WASI_file_statistics(const Xila_file_system_statistics_type *Statistics, __wasi_filestat_t *WASI_Statistics)
{
    printf("Statistics: %p\n", Statistics);

    WASI_Statistics->st_dev = Statistics->File_system;
    WASI_Statistics->st_ino = Statistics->Inode;
    WASI_Statistics->st_nlink = Statistics->Links;
    WASI_Statistics->st_size = Statistics->Size;
    WASI_Statistics->st_atim = Statistics->Last_access;
    WASI_Statistics->st_mtim = Statistics->Last_modification;
    WASI_Statistics->st_ctim = Statistics->Last_status_change;

    switch (Statistics->Type)
    {
    case Xila_file_system_type_file:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_REGULAR_FILE;
        break;
    case Xila_file_system_type_directory:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_DIRECTORY;
        break;
    case Xila_file_system_type_symbolic_link:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_SYMBOLIC_LINK;
        break;
    case Xila_file_system_type_character_device:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_CHARACTER_DEVICE;
        break;
    case Xila_file_system_type_block_device:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_BLOCK_DEVICE;
        break;
        break;
    case Xila_file_system_type_socket:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_SOCKET_DGRAM;
        break;
    default:
        WASI_Statistics->st_filetype = __WASI_FILETYPE_UNKNOWN;
        break;
    }
}

wasi_libc_file_access_mode Into_WASI_access_mode(Xila_file_system_mode_type Mode)
{

    if (Mode & Xila_file_system_mode_write_bit)
    {
        if (Mode & Xila_file_system_mode_read_bit)
        {
            return WASI_LIBC_ACCESS_MODE_READ_WRITE;
        }

        return WASI_LIBC_ACCESS_MODE_WRITE_ONLY;
    }

    return WASI_LIBC_ACCESS_MODE_READ_ONLY;
}