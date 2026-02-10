#include "../../../../modules/abi/xila.h"

#include "platform_api_extension.h"
#include "platform_common.h"
#include "platform_wasi_types.h"

XilaFileSystemWhence into_xila_whence(__wasi_whence_t whence) {
  switch (whence) {
  case __WASI_WHENCE_CUR:
    return XilaFileSystemWhence_Current;
  case __WASI_WHENCE_END:
    return XilaFileSystemWhence_End;
  default:
    return XilaFileSystemWhence_Start;
  }
}

__wasi_errno_t into_wasi_error(XilaFileSystemResult error) {
  switch (error) {
  case 0:
    return __WASI_ESUCCESS;
  default:
    return error;
    break;
  }
}

__wasi_filetype_t into_wasi_file_type(XilaFileKind type) {
  switch (type) {
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

void into_wasi_file_statistics(const XilaFileSystemStatistics *statistics,
                               __wasi_filestat_t *wasi_statistics) {
  wasi_statistics->st_dev = statistics->file_system;
  wasi_statistics->st_ino = statistics->inode;
  wasi_statistics->st_nlink = statistics->links;
  wasi_statistics->st_size = statistics->size;
  wasi_statistics->st_atim = statistics->access;
  wasi_statistics->st_mtim = statistics->modification;
  wasi_statistics->st_ctim = statistics->status;
  wasi_statistics->st_filetype = into_wasi_file_type(statistics->kind);
}

wasi_libc_file_access_mode into_wasi_access_mode(XilaFileSystemMode mode) {

  if (mode & XILA_FILE_SYSTEM_MODE_WRITE_MASK) {
    if (mode & XILA_FILE_SYSTEM_MODE_READ_MASK) {
      return WASI_LIBC_ACCESS_MODE_READ_WRITE;
    }

    return WASI_LIBC_ACCESS_MODE_WRITE_ONLY;
  }

  return WASI_LIBC_ACCESS_MODE_READ_ONLY;
}

XilaFileSystemMode into_xila_mode(wasi_libc_file_access_mode mode) {
  switch (mode) {
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

XilaFileSystemOpen into_xila_open(__wasi_oflags_t wasi_open) {
  XilaFileSystemOpen open = 0;

  if (wasi_open & __WASI_O_CREAT)
    open |= XILA_FILE_SYSTEM_OPEN_CREATE_MASK;

  if (wasi_open & __WASI_O_EXCL)
    open |= XILA_FILE_SYSTEM_OPEN_CREATE_ONLY_MASK;

  if (wasi_open & __WASI_O_TRUNC)
    open |= XILA_FILE_SYSTEM_OPEN_TRUNCATE_MASK;

  return open;
}

XilaFileSystemStatus into_xila_status(__wasi_fdflags_t wasi_status) {
  XilaFileSystemStatus status = 0;

  if (wasi_status & __WASI_FDFLAG_APPEND)
    status |= XILA_FILE_SYSTEM_STATUS_APPEND_MASK;

  if (wasi_status & __WASI_FDFLAG_SYNC)
    status |= XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_MASK;

  if (wasi_status & __WASI_FDFLAG_DSYNC)
    status |= XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_DATA_ONLY_MASK;

  if (wasi_status & __WASI_FDFLAG_NONBLOCK)
    status |= XILA_FILE_SYSTEM_STATUS_NON_BLOCKING_MASK;

  return status;
}

__wasi_fdflags_t into_wasi_status(XilaFileSystemStatus status) {
  __wasi_fdflags_t wasi_status = 0;

  if (status & XILA_FILE_SYSTEM_STATUS_APPEND_MASK)
    wasi_status |= __WASI_FDFLAG_APPEND;

  if (status & XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_MASK)
    wasi_status |= __WASI_FDFLAG_SYNC;

  if (status & XILA_FILE_SYSTEM_STATUS_SYNCHRONOUS_DATA_ONLY_MASK)
    wasi_status |= __WASI_FDFLAG_DSYNC;

  if (status & XILA_FILE_SYSTEM_STATUS_NON_BLOCKING_MASK)
    wasi_status |= __WASI_FDFLAG_NONBLOCK;

  return wasi_status;
}