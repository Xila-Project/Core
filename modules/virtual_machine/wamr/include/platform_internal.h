/*
 * Copyright (C) 2019 Intel Corporation.  All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

#ifndef _XILA_PLATFORM_INTERNAL_H
#define _XILA_PLATFORM_INTERNAL_H

#include "../../../abi/include/xila.h"

#include "stubs.h"
#include <stdint.h>
#include <stdarg.h>
#include <stdbool.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>
#include <errno.h>
#include <math.h>
#include <unistd.h>
//#include <pthread.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <sys/uio.h>
#include <poll.h>
#include <limits.h>
#include <assert.h>
#include <sys/ioctl.h>

#ifdef __cplusplus
extern "C"
{
#endif /* end of extern "C" */

#ifndef BH_PLATFORM_XILA
#define BH_PLATFORM_XILA
#endif

#define bh_socket_t size_t

#ifndef PATH_MAX
#define PATH_MAX 256
#endif

    typedef size_t korp_tid;

    struct RawMutex
    {
        uint8_t _[32];
    } __attribute__((aligned(8)));

    typedef struct RawMutex korp_mutex;

    typedef XilaConditionVariable korp_cond;
    typedef XilaThreadIdentifier korp_thread;

    struct RawRwLock
    {
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

    typedef uint64_t os_dir_stream;
    typedef uint64_t os_raw_file_handle;
    typedef uint64_t os_file_handle;

    uint64_t os_get_invalid_handle();

    int os_getpagesize();


#ifdef __cplusplus
}
#endif /* end of extern "C" */

#endif /* _XILA_PLATFORM_INTERNAL_H */
