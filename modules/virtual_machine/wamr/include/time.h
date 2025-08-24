#ifndef __XILA_TIME_H
#define __XILA_TIME_H

#include <stddef.h>
#include <stdint.h>

typedef uint64_t time_t;

struct timespec {
  time_t tv_sec; /* seconds */
  long tv_nsec;  /* nanoseconds */
};

typedef size_t clockid_t;

int clock_nanosleep(clockid_t clockid, int flags, const struct timespec *t,
                    struct timespec *remain);

#define TIMER_ABSTIME 0x01 /* Absolute time */

#define CLOCK_REALTIME 0 /* System-wide real-time clock */

#endif /* __XILA_TIME_H */