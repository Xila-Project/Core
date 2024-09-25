#include "../../../ABI/include/Xila.h"

wasi_libc_file_access_mode Into_WASI_access_mode(Xila_file_system_mode_type Mode);

Xila_file_system_mode_type Into_Xila_mode(wasi_libc_file_access_mode Mode);
Xila_file_system_open_type Into_Xila_open(__wasi_oflags_t WASI_open);
Xila_file_system_status_type Into_Xila_status(__wasi_fdflags_t WASI_status);