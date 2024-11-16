#include "Xila_graphics.h"

typedef uint16_t Function_call_type;

__attribute__((import_module("host")))
__attribute__((import_name("Xila_graphics_call"))) extern int
Xila_graphics_call(
    Function_call_type Function,
    size_t Argument_0,
    size_t Argument_1,
    size_t Argument_2,
    size_t Argument_3,
    size_t Argument_4,
    size_t Argument_5,
    size_t Argument_6,
    uint8_t Argument_count,
    void* Return_value
);
