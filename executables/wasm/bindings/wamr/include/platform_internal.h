#ifndef _XILA_PLATFORM_INTERNAL_H
#define _XILA_PLATFORM_INTERNAL_H

#include "wasm.generated.h"

// #include "stubs.h"
/* Safe Standard C Libraries */
//#include <assert.h>
//#include <ctype.h>
//#include <errno.h>
#include <limits.h>
//#include <math.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
//#include <stdio.h>
//#include <stdlib.h>
//#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif /* end of extern "C" */

#ifndef BH_PLATFORM_XILA
#define BH_PLATFORM_XILA
#endif

#define bh_socket_t XilaFileSystemItem *

#ifndef PATH_MAX
#define PATH_MAX 256
#endif

typedef size_t korp_tid;

struct RawMutex {
  uint8_t _[32];
} __attribute__((aligned(8)));

typedef struct RawMutex korp_mutex;

typedef XilaConditionVariable korp_cond;
typedef XilaTaskIdentifier korp_thread;


int xila_string_format(char *buf, size_t size, const char *format, ...);
int xila_string_format_varidic(char *buf, size_t size, const char *format, va_list args);

#define memcpy xila_memory_copy
#define memcmp xila_memory_compare
#define printf xila_print
#define memset xila_memory_set
#define strncpy xila_string_copy_bounded
#define abort() xila_panic("wasm runtime abort")
#define memmove xila_memory_move
#define strlen xila_string_get_length
#define abs xila_mathematics_absolute_value
#define isnan xila_mathematics_is_not_a_number
#define strncasecmp xila_string_compare_case_insensitive_bounded
#define strcmp xila_string_compare
#define strtod xila_string_to_double
#define strcmp xila_string_compare
#define strok_r xila_string_tokenize_reentrant
#define strtoul xila_string_to_unsigned_long
#define strtoull xila_string_to_unsigned_long_long

#define snprintf xila_string_format
#define vsnprintf xila_string_format_varidic

#define strtok_r xila_string_tokenize_reentrant
#define strtof xila_string_to_float
#define strncmp xila_string_compare_bounded
#define bsearch xila_binary_search
#define qsort xila_quick_sort
#define strstr xila_string_search
#define strtol xila_string_to_unsigned_long
#define strnlen xila_string_get_length_bounded

#define fabsf xila_mathematics_absolute_value_f32
#define ceilf xila_mathematics_ceiling_f32
#define floorf xila_mathematics_floor_f32
#define truncf xila_mathematics_truncate_f32
#define rintf xila_mathematics_round_f32
#define sqrtf xila_mathematics_square_root_f32

#define fabs xila_mathematics_absolute_value_f64
#define ceil xila_mathematics_ceiling_f64
#define floor xila_mathematics_floor_f64
#define trunc xila_mathematics_truncate_f64
#define rint xila_mathematics_round_f64
#define sqrt xila_mathematics_square_root_f64

#define signbit(x) ((x) < 0 ? -1 : ((x) > 0 ? 1 : 0))

#define unsigned signed
typedef __SIZE_TYPE__ ssize_t;
#undef unsigned


struct RawRwLock {
  uint8_t _[8];
} __attribute__((aligned(8)));

typedef struct RawRwLock korp_rwlock;

typedef struct XilaSemaphore korp_sem;

// #define OS_THREAD_MUTEX_INITIALIZER PTHREAD_MUTEX_INITIALIZER

#define BH_APPLET_PRESERVED_STACK_SIZE (2 * BH_KB)

/* Default thread priority */
#define BH_THREAD_DEFAULT_PRIORITY 5

/* Special value for tv_nsec field of timespec */

#define UTIME_NOW ((1l << 30) - 1l)
#ifndef __cplusplus
#define UTIME_OMIT ((1l << 30) - 2l)
#endif

#ifdef DT_UNKNOWN
#undef DT_UNKNOWN
#endif

#ifdef DT_REG
#undef DT_REG
#endif

#ifdef DT_DIR
#undef DT_DIR
#endif

/* Below parts of d_type define are ported from Nuttx, under Apache License v2.0
 */

/* File type code for the d_type field in dirent structure.
 * Note that because of the simplified filesystem organization of the NuttX,
 * top-level, pseudo-file system, an inode can be BOTH a file and a directory
 */

#define DTYPE_UNKNOWN 0
#define DTYPE_FIFO 1
#define DTYPE_CHR 2
#define DTYPE_SEM 3
#define DTYPE_DIRECTORY 4
#define DTYPE_MQ 5
#define DTYPE_BLK 6
#define DTYPE_SHM 7
#define DTYPE_FILE 8
#define DTYPE_MTD 9
#define DTYPE_LINK 10
#define DTYPE_SOCK 12

#define DT_UNKNOWN DTYPE_UNKNOWN
#define DT_FIFO DTYPE_FIFO
#define DT_CHR DTYPE_CHR
#define DT_SEM DTYPE_SEM
#define DT_DIR DTYPE_DIRECTORY
#define DT_MQ DTYPE_MQ
#define DT_BLK DTYPE_BLK
#define DT_SHM DTYPE_SHM
#define DT_REG DTYPE_FILE
#define DT_MTD DTYPE_MTD
#define DT_LNK DTYPE_LINK
#define DT_SOCK DTYPE_SOCK

typedef struct timespec os_timespec;

typedef XilaFileSystemItem *os_dir_stream;
typedef XilaFileSystemItem *os_raw_file_handle;
typedef XilaFileSystemItem *os_file_handle;

typedef os_file_handle os_nfds_t;

typedef struct {
  os_nfds_t fd;
  short revents;
  short events;
} os_poll_file_handle;

os_file_handle os_get_invalid_handle();

int os_getpagesize();

#ifdef __cplusplus
}
#endif /* end of extern "C" */

#endif /* _XILA_PLATFORM_INTERNAL_H */
