#include "../../../abi/include/xila.h"

#include "platform_api_extension.h"
#include "platform_common.h"
#include "platform_wasi_types.h"

wasi_libc_file_access_mode into_wasi_access_mode(XilaFileSystemMode Mode);

XilaFileSystemMode into_xila_mode(wasi_libc_file_access_mode Mode);
XilaFileSystemOpen into_xila_open(__wasi_oflags_t WASI_open);
XilaFileSystemStatus into_xila_status(__wasi_fdflags_t WASI_status);
__wasi_errno_t into_wasi_error(XilaFileSystemResult Error);
__wasi_whence_t into_xila_whence(XilaFileSystemWhence Whence);

__wasi_filetype_t into_wasi_file_type(XilaFileKind Type);
void into_wasi_file_statistics(const XilaFileSystemStatistics *statistics,
                               __wasi_filestat_t *wasi_statistics);
__wasi_fdflags_t into_wasi_fdflags(XilaFileSystemStatus status);