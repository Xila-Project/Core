#include "../../../ABI/include/Xila.h"

#include "platform_api_extension.h"
#include "platform_wasi_types.h"
#include "platform_common.h"

Xila_file_system_whence_type Into_Xila_whence(__wasi_whence_t Whence)
{
    switch (Whence)
    {
    case __WASI_WHENCE_CUR:
        return Xila_file_system_whence_type_Current;
    case __WASI_WHENCE_END:
        return Xila_file_system_whence_type_End;
    default:
        return Xila_file_system_whence_type_Start;
    }
}

__wasi_errno_t Into_WASI_Error(Xila_file_system_result_type Error)
{
    switch (Error)
    {
    case 0:
        return __WASI_ESUCCESS;
    default:
        return Error;
        break;
    }
}

__wasi_filetype_t Into_WASI_file_type(Xila_file_type_type Type)
{
    switch (Type)
    {
    case Xila_file_type_type_File:
        return __WASI_FILETYPE_REGULAR_FILE;
    case Xila_file_type_type_Directory:
        return __WASI_FILETYPE_DIRECTORY;
    case Xila_file_type_type_Symbolic_link:
        return __WASI_FILETYPE_SYMBOLIC_LINK;
    case Xila_file_type_type_Character_device:
        return __WASI_FILETYPE_CHARACTER_DEVICE;
    case Xila_file_type_type_Block_device:
        return __WASI_FILETYPE_BLOCK_DEVICE;
    case Xila_file_type_type_Socket:
        return __WASI_FILETYPE_SOCKET_DGRAM;
    default:
        return __WASI_FILETYPE_UNKNOWN;
    }
}

void Into_WASI_file_statistics(const Xila_file_system_statistics_type *Statistics, __wasi_filestat_t *WASI_Statistics)
{
    WASI_Statistics->st_dev = Statistics->file_system;
    WASI_Statistics->st_ino = Statistics->inode;
    WASI_Statistics->st_nlink = Statistics->links;
    WASI_Statistics->st_size = Statistics->size;
    WASI_Statistics->st_atim = Statistics->last_access;
    WASI_Statistics->st_mtim = Statistics->last_modification;
    WASI_Statistics->st_ctim = Statistics->last_status_change;
    WASI_Statistics->st_filetype = Into_WASI_file_type(Statistics->Type);
}

wasi_libc_file_access_mode Into_WASI_access_mode(Xila_file_system_mode_type Mode)
{

    if (Mode & XILA_FILE_SYSTEM_MODE_WRITE_MASK)
    {
        if (Mode & XILA_FILE_SYSTEM_MODE_READ_MASK)
        {
            return WASI_LIBC_ACCESS_MODE_READ_WRITE;
        }

        return WASI_LIBC_ACCESS_MODE_WRITE_ONLY;
    }

    return WASI_LIBC_ACCESS_MODE_READ_ONLY;
}

Xila_file_system_mode_type Into_Xila_mode(wasi_libc_file_access_mode Mode)
{
    switch (Mode)
    {
    case WASI_LIBC_ACCESS_MODE_READ_ONLY:
        return XILA_FILE_SYSTEM_MODE_READ_MASK;

    case WASI_LIBC_ACCESS_MODE_WRITE_ONLY:
        return XILA_FILE_SYSTEM_MODE_WRITE_MASK;
    case WASI_LIBC_ACCESS_MODE_READ_WRITE:
        return XILA_FILE_SYSTEM_MODE_READ_MASK | XILA_FILE_SYSTEM_MODE_WRITE_MASK;
    default:
        return 0;
    }
}

Xila_file_system_open_type Into_Xila_open(__wasi_oflags_t WASI_open)
{
    Xila_file_system_open_type Open = 0;

    if (WASI_open & __WASI_O_CREAT)
        Open |= XILA_FILE_SYSTEM_OPEN_CREATE_MASK;

    if (WASI_open & __WASI_O_EXCL)
        Open |= XILA_FILE_SYSTEM_OPEN_CREATE_ONLY_MASK;

    if (WASI_open & __WASI_O_TRUNC)
        Open |= XILA_FILE_SYSTEM_OPEN_TRUNCATE_MASK;

    return Open;
}

Xila_file_system_status_type Into_Xila_status(__wasi_fdflags_t WASI_status)
{
    Xila_file_system_status_type Status = 0;

    if (WASI_status & __WASI_FDFLAG_APPEND)
        Status |= XILA_FILE_SYSTEM_STATUS_APPEND_MASK;

    if (WASI_status & __WASI_FDFLAG_SYNC)
        Status |= XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_MASK;

    if (WASI_status & __WASI_FDFLAG_DSYNC)
        Status |= XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_DATA_ONLY_MASK;

    if (WASI_status & __WASI_FDFLAG_NONBLOCK)
        Status |= XILA_FILE_SYSTEM_STATUS_NON_BLOCKING_MASK;

    return Status;
}