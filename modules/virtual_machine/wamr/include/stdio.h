#ifndef __XILA_STDIO_H
#define __XILA_STDIO_H

#include <stdarg.h>
#include <stddef.h>

int vprintf(const char *format, va_list arg);
int vsnprintf(char *str, size_t size, const char *format, va_list arg);

int open(const char *pathname, int flags, ...);
int read(int fd, void *buf, size_t count);

#define O_RDONLY 0x0000
#define O_WRONLY 0x0001
#define O_RDWR   0x0002

#endif /* __XILA_STDIO_H */