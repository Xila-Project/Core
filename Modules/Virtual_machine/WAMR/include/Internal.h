#include "../../../ABI/include/Xila.h"

#include "platform_api_extension.h"
#include "platform_wasi_types.h"
#include "platform_common.h"

wasi_libc_file_access_mode Into_WASI_access_mode(Xila_file_system_mode_type Mode);

Xila_file_system_mode_type Into_Xila_mode(wasi_libc_file_access_mode Mode);
Xila_file_system_open_type Into_Xila_open(__wasi_oflags_t WASI_open);
Xila_file_system_status_type Into_Xila_status(__wasi_fdflags_t WASI_status);
__wasi_errno_t Into_WASI_Error(Xila_file_system_result_type Error);
__wasi_whence_t Into_Xila_whence(Xila_file_system_whence_type Whence);
__wasi_filetype_t Into_WASI_file_type(Xila_file_type_type Type);