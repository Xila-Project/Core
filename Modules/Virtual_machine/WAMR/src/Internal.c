#include "../../../ABI/include/xila.h"

#include "platform_api_extension.h"
#include "platform_common.h"
#include "platform_wasi_types.h"

XilaFileSystemWhence Into_xila_whence(__wasi_whence_t Whence) {
  switch (Whence) {
  case __WASI_WHENCE_CUR:
    return XilaFileSystemWhence_Current;
  case __WASI_WHENCE_END:
    return XilaFileSystemWhence_End;
  default:
    return XilaFileSystemWhence_Start;
  }
}

__wasi_errno_t Into_WASI_Error(XilaFileSystemResult Error) {
  switch (Error) {
  case 0:
    return __WASI_ESUCCESS;
  default:
    return Error;
    break;
  }
}

__wasi_filetype_t Into_WASI_file_type(XilaFileKind Type) {
  switch (Type) {
  case XilaFileKind_File:
    return __WASI_FILETYPE_REGULAR_FILE;
  case XilaFileKind_Directory:
    return __WASI_FILETYPE_DIRECTORY;
  case XilaFileKind_SymbolicLink:
    return __WASI_FILETYPE_SYMBOLIC_LINK;
  case XilaFileKind_CharacterDevice:
    return __WASI_FILETYPE_CHARACTER_DEVICE;
  case XilaFileKind_BlockDevice:
    return __WASI_FILETYPE_BLOCK_DEVICE;
  case XilaFileKind_Socket:
    return __WASI_FILETYPE_SOCKET_DGRAM;
  default:
    return __WASI_FILETYPE_UNKNOWN;
  }
}

void Into_WASI_file_statistics(
    const XilaFileSystemStatistics *Statistics,
    __wasi_filestat_t *WASI_Statistics) {
  WASI_Statistics->st_dev = Statistics->file_system;
  WASI_Statistics->st_ino = Statistics->inode;
  WASI_Statistics->st_nlink = Statistics->links;
  WASI_Statistics->st_size = Statistics->size;
  WASI_Statistics->st_atim = Statistics->last_access;
  WASI_Statistics->st_mtim = Statistics->last_modification;
  WASI_Statistics->st_ctim = Statistics->last_status_change;
  WASI_Statistics->st_filetype = Into_WASI_file_type(Statistics->type);
}

wasi_libc_file_access_mode
Into_WASI_access_mode(XilaFileSystemMode Mode) {

  if (Mode & XILA_FILE_SYSTEM_MODE_WRITE_MASK) {
    if (Mode & XILA_FILE_SYSTEM_MODE_READ_MASK) {
      return WASI_LIBC_ACCESS_MODE_READ_WRITE;
    }

    return WASI_LIBC_ACCESS_MODE_WRITE_ONLY;
  }

  return WASI_LIBC_ACCESS_MODE_READ_ONLY;
}

XilaFileSystemMode Into_xila_mode(wasi_libc_file_access_mode Mode) {
  switch (Mode) {
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

XilaFileSystemOpen Into_xila_open(__wasi_oflags_t WASI_open) {
  XilaFileSystemOpen Open = 0;

  if (WASI_open & __WASI_O_CREAT)
    Open |= XILA_FILE_SYSTEM_OPEN_CREATE_MASK;

  if (WASI_open & __WASI_O_EXCL)
    Open |= XILA_FILE_SYSTEM_OPEN_CREATE_ONLY_MASK;

  if (WASI_open & __WASI_O_TRUNC)
    Open |= XILA_FILE_SYSTEM_OPEN_TRUNCATE_MASK;

  return Open;
}

XilaFileSystemStatus Into_xila_status(__wasi_fdflags_t WASI_status) {
  XilaFileSystemStatus Status = 0;

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