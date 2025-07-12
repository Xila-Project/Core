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

// Coordinate utility functions
int32_t Xila_graphics_coord_type(int32_t x) {
  return x & XILA_GRAPHICS_COORD_TYPE_MASK;
}

int32_t Xila_graphics_coord_plain(int32_t x) {
  return x & ~XILA_GRAPHICS_COORD_TYPE_MASK;
}

bool Xila_graphics_coord_is_px(int32_t x) {
  return Xila_graphics_coord_type(x) == XILA_GRAPHICS_COORD_TYPE_PX || 
  Xila_graphics_coord_type(x) == XILA_GRAPHICS_COORD_TYPE_PX_NEG;
}

bool Xila_graphics_coord_is_spec(int32_t x) {
  return Xila_graphics_coord_type(x) == XILA_GRAPHICS_COORD_TYPE_SPEC;
}

int32_t Xila_graphics_coord_set_spec(int32_t x) {
  return x | XILA_GRAPHICS_COORD_TYPE_SPEC;
}

int32_t Xila_graphics_max(int32_t a, int32_t b) {
  return (a > b) ? a : b;
}

int32_t Xila_graphics_min(int32_t a, int32_t b) {
  return (a < b) ? a : b;
}

int32_t Xila_graphics_pct(int32_t x) {
  if (x < 0) {
    return Xila_graphics_coord_set_spec(
      XILA_GRAPHICS_PCT_POS_MAX - Xila_graphics_max(x, -XILA_GRAPHICS_PCT_POS_MAX)
    );
  } else {
    return Xila_graphics_coord_set_spec(
      Xila_graphics_min(x, XILA_GRAPHICS_PCT_POS_MAX)
    );
  }
}

bool Xila_graphics_coord_is_pct(int32_t x) {
  return Xila_graphics_coord_is_spec(x) && 
  Xila_graphics_coord_plain(x) <= XILA_GRAPHICS_PCT_STORED_MAX;
}

int32_t Xila_graphics_coord_get_pct(int32_t x) {
  int32_t plain = Xila_graphics_coord_plain(x);
  return plain > XILA_GRAPHICS_PCT_POS_MAX ? 
  XILA_GRAPHICS_PCT_POS_MAX - plain : plain;
}

int32_t Xila_graphics_size_content(void) {
    return XILA_GRAPHICS_SIZE_CONTENT;
}
