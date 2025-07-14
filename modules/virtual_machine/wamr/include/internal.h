#include "../../../abi/include/xila.h"

#include "platform_api_extension.h"
#include "platform_wasi_types.h"
#include "platform_common.h"

wasi_libc_file_access_mode Into_WASI_access_mode(XilaFileSystemMode Mode);

XilaFileSystemMode Into_xila_mode(wasi_libc_file_access_mode Mode);
XilaFileSystemOpen Into_xila_open(__wasi_oflags_t WASI_open);
XilaFileSystemStatus Into_xila_status(__wasi_fdflags_t WASI_status);
__wasi_errno_t Into_WASI_Error(XilaFileSystemResult Error);
__wasi_whence_t Into_xila_whence(XilaFileSystemWhence Whence);
__wasi_filetype_t Into_WASI_file_type(XilaFileKind Type);