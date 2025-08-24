#include "__xila_abi_generated.h"

// POSIX-compliant function name mappings to descriptive Xila functions

#define PRIu8 "u"
#define PRIu32 "u"
#define PRIx16 "x"
#define PRId16 "d"
#define PRIx32 "x"
#define PRId32 "d"

#define FIONREAD 0x541B

#ifndef NULL
#define NULL ((void *)0)
#endif

// Memory functions
#define memcpy xila_memory_copy
#define memset xila_memory_set
#define memcmp xila_memory_compare
#define memmove xila_memory_move

// String length functions
#define strlen xila_string_get_length
#define strnlen xila_string_get_length_bounded

// String comparison functions
#define strcmp xila_string_compare
#define strncmp xila_string_compare_bounded
#define strncasecmp xila_string_compare_case_insensitive_bounded

// String copy functions
#define strncpy xila_string_copy_bounded

// String search functions
#define strstr xila_string_find_substring
#define strchr xila_string_find_character
#define strcspn xila_string_span_complement
#define strspn xila_string_span

// String conversion functions
#define strtod xila_string_to_double
#define strtof xila_string_to_float
#define strtoul xila_string_to_unsigned_long
#define strtoull xila_string_to_unsigned_long_long

// String tokenization functions
#define strtok xila_string_tokenize

#define isnan xila_is_nan

#define abs xila_get_absolute_value

// Wrapper function for abort to provide WAMR context
static inline void xila_abort_wrapper(void) {
    xila_abort("WAMR");
}

#define abort xila_abort_wrapper

#define qsort xila_sort_quick
#define bsearch xila_search_binary

#define atoi xila_string_parse_integer

int ioctl(int fd, unsigned long op, ...);

int sched_yield();

int snprintf(char *s, size_t n, const char *format, ...);