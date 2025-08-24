

typedef struct {
  int __data;
} pthread_once_t;

int pthread_once(pthread_once_t *once_control, void (*init_routine)(void));

#define PTHREAD_ONCE_INIT { 0 }