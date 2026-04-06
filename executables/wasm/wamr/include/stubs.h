#ifndef _XILA_WAMR_STUBS_H
#define _XILA_WAMR_STUBS_H

#include "../../../../modules/abi/xila.h"

#ifndef NULL
#define NULL ((void *)0)
#endif

#ifndef __clockid_t_defined
#define __clockid_t_defined 1
typedef XilaTimeClockIdentifier clockid_t;
#endif

#ifndef __time_t_defined
#define __time_t_defined 1
typedef long int time_t;
#endif

#ifndef _STRUCT_TIMESPEC
#define _STRUCT_TIMESPEC
struct timespec {
  time_t tv_sec;
  long tv_nsec;
};
#endif

#ifndef _BITS_PTHREADTYPES_COMMON_H
#define _BITS_PTHREADTYPES_COMMON_H 1
typedef XilaConditionVariableAttribute pthread_condattr_t;
#endif

struct pollfd {
  XilaFileIdentifier fd;
  short events;
  short revents;
};

#define POLLIN 0x0001
#define POLLOUT 0x0004
#define POLLERR 0x0008
#define POLLHUP 0x0010
#define POLLNVAL 0x0020

#define FIONREAD 0x541B

#define CLOCK_REALTIME XilaTimeClockIdentifier_Realtime
#define CLOCK_MONOTONIC XilaTimeClockIdentifier_Monotonic

#define TIMER_ABSTIME XilaTimerFlags_Absolute

#define pthread_cond_init(cond, attr)                                         \
  xila_condition_variable_initialize(                                         \
      (XilaConditionVariable *)(cond),                                        \
      (const XilaConditionVariableAttribute *)(attr))

#define pthread_condattr_init(attr)                                            \
  xila_condition_variable_attribute_initialize(                                \
      (XilaConditionVariableAttribute *)(attr))

#define pthread_condattr_setclock(attr, clock)                                \
  xila_condition_variable_attribute_set_clock(                                \
      (XilaConditionVariableAttribute *)(attr),                               \
      (XilaTimeClockIdentifier)(clock))

#define pthread_condattr_destroy(attr)                                         \
  xila_condition_variable_attribute_destroy(                                   \
      (XilaConditionVariableAttribute *)(attr))

static inline int clock_nanosleep(clockid_t clock_id, int flags,
                                  const struct timespec *req,
                                  struct timespec *rem) {
  XilaTime request = 0;
  XilaTime remaining = 0;

  if (req != NULL) {
    request = ((XilaTime)req->tv_sec * 1000000000ULL) + (XilaTime)req->tv_nsec;
  }

  int result = xila_time_nano_sleep((XilaTimeClockIdentifier)clock_id,
                                    (XilaTimerFlags)flags,
                                    (const void *)&request,
                                    (XilaTime *)&remaining);

  if (rem != NULL) {
    rem->tv_sec = (time_t)(remaining / 1000000000ULL);
    rem->tv_nsec = (long)(remaining % 1000000000ULL);
  }

  return result;
}

#define sched_yield xila_thread_yield

static inline uint32_t xila_htonl(uint32_t value) {
  return __builtin_bswap32(value);
}

static inline uint16_t xila_htons(uint16_t value) {
  return __builtin_bswap16(value);
}

#define htonl xila_htonl
#define htons xila_htons

#endif /* _XILA_WAMR_STUBS_H */
