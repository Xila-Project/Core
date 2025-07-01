

typedef struct {
  int32_t x;
  int32_t y;
} Xila_graphics_point_t;

typedef struct {
  int32_t x1;
  int32_t y1;
  int32_t x2;
  int32_t y2;
} Xila_graphics_area_t;

typedef struct {
  uint8_t blue;
  uint8_t green;
  uint8_t red;
} Xila_graphics_color_t;

typedef struct {
  uint16_t blue : 5;
  uint16_t green : 6;
  uint16_t red : 5;
} Xila_graphics_color16_t;

typedef struct {
  uint8_t blue;
  uint8_t green;
  uint8_t red;
  uint8_t alpha;
} Xila_graphics_color32_t;

typedef struct {
  uint16_t h;
  uint8_t s;
  uint8_t v;
} Xila_graphics_color_hsv_t;

typedef struct {
  uint8_t lumi;
  uint8_t alpha;
} Xila_graphics_color16a_t;

#define XILA_GRAPHICS_COLOR_MAKE(r8, g8, b8) {b8, g8, r8}

#define XILA_GRAPHICS_OPA_MIX2(a1, a2) (((int32_t)(a1) * (a2)) >> 8)
#define XILA_GRAPHICS_OPA_MIX3(a1, a2, a3) (((int32_t)(a1) * (a2) * (a3)) >> 16)

typedef int32_t Xila_graphics_value_precise_t;

typedef struct {
  Xila_graphics_value_precise_t x;
  Xila_graphics_value_precise_t y;
} Xila_graphics_point_precise_t;

typedef enum {
  XILA_GRAPHICS_COLOR_FORMAT_UNKNOWN = 0,

  XILA_GRAPHICS_COLOR_FORMAT_RAW = 0x01,
  XILA_GRAPHICS_COLOR_FORMAT_RAW_ALPHA = 0x02,

  /*<=1 byte (+alpha) formats*/
  XILA_GRAPHICS_COLOR_FORMAT_L8 = 0x06,
  XILA_GRAPHICS_COLOR_FORMAT_I1 = 0x07,
  XILA_GRAPHICS_COLOR_FORMAT_I2 = 0x08,
  XILA_GRAPHICS_COLOR_FORMAT_I4 = 0x09,
  XILA_GRAPHICS_COLOR_FORMAT_I8 = 0x0A,
  XILA_GRAPHICS_COLOR_FORMAT_A8 = 0x0E,

  /*2 byte (+alpha) formats*/
  XILA_GRAPHICS_COLOR_FORMAT_RGB565 = 0x12,
  XILA_GRAPHICS_COLOR_FORMAT_ARGB8565 =
      0x13, /**< Not supported by sw renderer yet. */
  XILA_GRAPHICS_COLOR_FORMAT_RGB565A8 =
      0x14, /**< Color array followed by Alpha array*/
  XILA_GRAPHICS_COLOR_FORMAT_AL88 = 0x15, /**< L8 with alpha >*/

  /*3 byte (+alpha) formats*/
  XILA_GRAPHICS_COLOR_FORMAT_RGB888 = 0x0F,
  XILA_GRAPHICS_COLOR_FORMAT_ARGB8888 = 0x10,
  XILA_GRAPHICS_COLOR_FORMAT_XRGB8888 = 0x11,

  /*Formats not supported by software renderer but kept here so GPU can use it*/
  XILA_GRAPHICS_COLOR_FORMAT_A1 = 0x0B,
  XILA_GRAPHICS_COLOR_FORMAT_A2 = 0x0C,
  XILA_GRAPHICS_COLOR_FORMAT_A4 = 0x0D,

  /* reference to https://wiki.videolan.org/YUV/ */
  /*YUV planar formats*/
  XILA_GRAPHICS_COLOR_FORMAT_YUV_START = 0x20,
  XILA_GRAPHICS_COLOR_FORMAT_I420 =
      XILA_GRAPHICS_COLOR_FORMAT_YUV_START, /*YUV420 planar(3 plane)*/
  XILA_GRAPHICS_COLOR_FORMAT_I422 = 0x21,   /*YUV422 planar(3 plane)*/
  XILA_GRAPHICS_COLOR_FORMAT_I444 = 0x22,   /*YUV444 planar(3 plane)*/
  XILA_GRAPHICS_COLOR_FORMAT_I400 = 0x23,   /*YUV400 no chroma channel*/
  XILA_GRAPHICS_COLOR_FORMAT_NV21 =
      0x24, /*YUV420 planar(2 plane), UV plane in 'V, U, V, U'*/
  XILA_GRAPHICS_COLOR_FORMAT_NV12 =
      0x25, /*YUV420 planar(2 plane), UV plane in 'U, V, U, V'*/

  /*YUV packed formats*/
  XILA_GRAPHICS_COLOR_FORMAT_YUY2 = 0x26, /*YUV422 packed like 'Y U Y V'*/
  XILA_GRAPHICS_COLOR_FORMAT_UYVY = 0x27, /*YUV422 packed like 'U Y V Y'*/

  XILA_GRAPHICS_COLOR_FORMAT_YUV_END = XILA_GRAPHICS_COLOR_FORMAT_UYVY,

} Xila_graphics_color_format_t;

typedef enum {
  XILA_GRAPHICS_FLEX_ALIGN_START,
  XILA_GRAPHICS_FLEX_ALIGN_END,
  XILA_GRAPHICS_FLEX_ALIGN_CENTER,
  XILA_GRAPHICS_FLEX_ALIGN_SPACE_EVENLY,
  XILA_GRAPHICS_FLEX_ALIGN_SPACE_AROUND,
  XILA_GRAPHICS_FLEX_ALIGN_SPACE_BETWEEN,
} Xila_graphics_flex_align_t;

#define XILA_GRAPHICS_FLEX_COLUMN (1 << 0)
#define XILA_GRAPHICS_FLEX_WRAP (1 << 2)
#define XILA_GRAPHICS_FLEX_REVERSE (1 << 3)

typedef uint8_t Xila_graphics_style_prop_t;

typedef uint8_t Xila_graphics_opa_t;

typedef enum {
  XILA_GRAPHICS_FLEX_FLOW_ROW = 0x00,
  XILA_GRAPHICS_FLEX_FLOW_COLUMN = XILA_GRAPHICS_FLEX_COLUMN,
  XILA_GRAPHICS_FLEX_FLOW_ROW_WRAP =
      XILA_GRAPHICS_FLEX_FLOW_ROW | XILA_GRAPHICS_FLEX_WRAP,
  XILA_GRAPHICS_FLEX_FLOW_ROW_REVERSE =
      XILA_GRAPHICS_FLEX_FLOW_ROW | XILA_GRAPHICS_FLEX_REVERSE,
  XILA_GRAPHICS_FLEX_FLOW_ROW_WRAP_REVERSE = XILA_GRAPHICS_FLEX_FLOW_ROW |
                                             XILA_GRAPHICS_FLEX_WRAP |
                                             XILA_GRAPHICS_FLEX_REVERSE,
  XILA_GRAPHICS_FLEX_FLOW_COLUMN_WRAP =
      XILA_GRAPHICS_FLEX_FLOW_COLUMN | XILA_GRAPHICS_FLEX_WRAP,
  XILA_GRAPHICS_FLEX_FLOW_COLUMN_REVERSE =
      XILA_GRAPHICS_FLEX_FLOW_COLUMN | XILA_GRAPHICS_FLEX_REVERSE,
  XILA_GRAPHICS_FLEX_FLOW_COLUMN_WRAP_REVERSE = XILA_GRAPHICS_FLEX_FLOW_COLUMN |
                                                XILA_GRAPHICS_FLEX_WRAP |
                                                XILA_GRAPHICS_FLEX_REVERSE,
} Xila_graphics_flex_flow_t;

typedef enum {
  XILA_GRAPHICS_GRAD_DIR_NONE, /**< No gradient (the `grad_color` property is
                                  ignored)*/
  XILA_GRAPHICS_GRAD_DIR_VER,  /**< Simple vertical (top to bottom) gradient*/
  XILA_GRAPHICS_GRAD_DIR_HOR,  /**< Simple horizontal (left to right) gradient*/
  XILA_GRAPHICS_GRAD_DIR_LINEAR,  /**< Linear gradient defined by start and end
                                     points. Can be at any angle.*/
  XILA_GRAPHICS_GRAD_DIR_RADIAL,  /**< Radial gradient defined by start and end
                                     circles*/
  XILA_GRAPHICS_GRAD_DIR_CONICAL, /**< Conical gradient defined by center point,
                                     start and end angles*/
} Xila_graphics_grad_dir_t;

/**
 * Possible options for blending opaque drawings
 */
typedef enum {
  XILA_GRAPHICS_BLEND_MODE_NORMAL,      /**< Simply mix according to the opacity
                                           value*/
  XILA_GRAPHICS_BLEND_MODE_ADDITIVE,    /**< Add the respective color channels*/
  XILA_GRAPHICS_BLEND_MODE_SUBTRACTIVE, /**< Subtract the foreground from the
                                           background*/
  XILA_GRAPHICS_BLEND_MODE_MULTIPLY,    /**< Multiply the foreground and
                                           background*/
} Xila_graphics_blend_mode_t;

/**
 * Some options to apply decorations on texts.
 * 'OR'ed values can be used.
 */
typedef enum {
  XILA_GRAPHICS_TEXT_DECOR_NONE = 0x00,
  XILA_GRAPHICS_TEXT_DECOR_UNDERLINE = 0x01,
  XILA_GRAPHICS_TEXT_DECOR_STRIKETHROUGH = 0x02,
} Xila_graphics_text_decor_t;

/**
 * Selects on which sides border should be drawn
 * 'OR'ed values can be used.
 */
typedef enum {
  XILA_GRAPHICS_BORDER_SIDE_NONE = 0x00,
  XILA_GRAPHICS_BORDER_SIDE_BOTTOM = 0x01,
  XILA_GRAPHICS_BORDER_SIDE_TOP = 0x02,
  XILA_GRAPHICS_BORDER_SIDE_LEFT = 0x04,
  XILA_GRAPHICS_BORDER_SIDE_RIGHT = 0x08,
  XILA_GRAPHICS_BORDER_SIDE_FULL = 0x0F,
  XILA_GRAPHICS_BORDER_SIDE_INTERNAL =
      0x10, /**< FOR matrix-like objects (e.g. Button matrix)*/
} Xila_graphics_border_side_t;

typedef uint8_t Xila_graphics_anim_t;

typedef enum {
  XILA_GRAPHICS_STYLE_STATE_CMP_SAME, /**< The style properties in the 2 states
                                         are identical */
  XILA_GRAPHICS_STYLE_STATE_CMP_DIFF_REDRAW,   /**< The differences can be shown
                                                  with a simple redraw */
  XILA_GRAPHICS_STYLE_STATE_CMP_DIFF_DRAW_PAD, /**< The differences can be shown
                                                  with a simple redraw */
  XILA_GRAPHICS_STYLE_STATE_CMP_DIFF_LAYOUT,   /**< The differences can be shown
                                                  with a simple redraw */
} Xila_graphics_style_state_cmp_t;

typedef uint32_t Xila_graphics_style_selector_t;

typedef uint16_t Xila_graphics_object_t;

typedef uint16_t Xila_graphics__lv_object_t;

typedef uint32_t Xila_graphics_part_t;

typedef struct {
  int32_t *x_points;
  int32_t *y_points;
  Xila_graphics_color_t color;
  uint32_t start_point;
  uint32_t hidden : 1;
  uint32_t x_ext_buf_assigned : 1;
  uint32_t y_ext_buf_assigned : 1;
  uint32_t x_axis_sec : 1;
  uint32_t y_axis_sec : 1;
} Xila_graphics_chart_series_t;

typedef enum {
  XILA_GRAPHICS_CHART_TYPE_NONE,    /**< Don't draw the series*/
  XILA_GRAPHICS_CHART_TYPE_LINE,    /**< Connect the points with lines*/
  XILA_GRAPHICS_CHART_TYPE_BAR,     /**< Draw columns*/
  XILA_GRAPHICS_CHART_TYPE_SCATTER, /**< Draw points and lines in 2D (x,y
                                       coordinates)*/
} lv_chart_type_t;

typedef void *Xila_graphics_style_t;

typedef struct {
} Xila_graphics_font_t;

typedef enum {
  XILA_GRAPHICS_DIR_NONE = 0x00,
  XILA_GRAPHICS_DIR_LEFT = (1 << 0),
  XILA_GRAPHICS_DIR_RIGHT = (1 << 1),
  XILA_GRAPHICS_DIR_TOP = (1 << 2),
  XILA_GRAPHICS_DIR_BOTTOM = (1 << 3),
  XILA_GRAPHICS_DIR_HOR = XILA_GRAPHICS_DIR_LEFT | XILA_GRAPHICS_DIR_RIGHT,
  XILA_GRAPHICS_DIR_VER = XILA_GRAPHICS_DIR_TOP | XILA_GRAPHICS_DIR_BOTTOM,
  XILA_GRAPHICS_DIR_ALL = XILA_GRAPHICS_DIR_HOR | XILA_GRAPHICS_DIR_VER,
} Xila_graphics_dir_t;

/**
 * Represents a date on the calendar object (platform-agnostic).
 */
typedef struct {
  uint16_t year;
  int8_t month; /**< 1..12 */
  int8_t day;   /**< 1..31 */
} Xila_graphics_calendar_date_t;

typedef enum {
  XILA_GRAPHICS_ALIGN_DEFAULT = 0,
  XILA_GRAPHICS_ALIGN_TOP_LEFT,
  XILA_GRAPHICS_ALIGN_TOP_MID,
  XILA_GRAPHICS_ALIGN_TOP_RIGHT,
  XILA_GRAPHICS_ALIGN_BOTTOM_LEFT,
  XILA_GRAPHICS_ALIGN_BOTTOM_MID,
  XILA_GRAPHICS_ALIGN_BOTTOM_RIGHT,
  XILA_GRAPHICS_ALIGN_LEFT_MID,
  XILA_GRAPHICS_ALIGN_RIGHT_MID,
  XILA_GRAPHICS_ALIGN_CENTER,

  XILA_GRAPHICS_ALIGN_OUT_TOP_LEFT,
  XILA_GRAPHICS_ALIGN_OUT_TOP_MID,
  XILA_GRAPHICS_ALIGN_OUT_TOP_RIGHT,
  XILA_GRAPHICS_ALIGN_OUT_BOTTOM_LEFT,
  XILA_GRAPHICS_ALIGN_OUT_BOTTOM_MID,
  XILA_GRAPHICS_ALIGN_OUT_BOTTOM_RIGHT,
  XILA_GRAPHICS_ALIGN_OUT_LEFT_TOP,
  XILA_GRAPHICS_ALIGN_OUT_LEFT_MID,
  XILA_GRAPHICS_ALIGN_OUT_LEFT_BOTTOM,
  XILA_GRAPHICS_ALIGN_OUT_RIGHT_TOP,
  XILA_GRAPHICS_ALIGN_OUT_RIGHT_MID,
  XILA_GRAPHICS_ALIGN_OUT_RIGHT_BOTTOM,
} Xila_graphics_align_t;

typedef enum {
  XILA_GRAPHICS_GRID_ALIGN_START,
  XILA_GRAPHICS_GRID_ALIGN_CENTER,
  XILA_GRAPHICS_GRID_ALIGN_END,
  XILA_GRAPHICS_GRID_ALIGN_STRETCH,
  XILA_GRAPHICS_GRID_ALIGN_SPACE_EVENLY,
  XILA_GRAPHICS_GRID_ALIGN_SPACE_AROUND,
  XILA_GRAPHICS_GRID_ALIGN_SPACE_BETWEEN,
} Xila_graphics_grid_align_t;

typedef uint16_t Xila_graphics_object_class_t;

/** Type to store button control bits (disabled, hidden etc.)
 * The first 3 bits are used to store the width*/
typedef enum {
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_HIDDEN = 0x0010, /**< Button hidden*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_NO_REPEAT =
      0x0020, /**< Do not repeat press this button.*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_DISABLED = 0x0040, /**< Disable this button.*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_CHECKABLE =
      0x0080, /**< The button can be toggled.*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_CHECKED =
      0x0100, /**< Button is currently toggled (e.g. checked).*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_CLICK_TRIG =
      0x0200, /**< 1: Send XILA_GRAPHICS_EVENT_VALUE_CHANGE on CLICK, 0: Send
                 XILA_GRAPHICS_EVENT_VALUE_CHANGE on PRESS*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_POPOVER =
      0x0400, /**< Show a popover when pressing this key*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_RESERVED_1 =
      0x0800, /**< Reserved for later use*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_RESERVED_2 =
      0x1000, /**< Reserved for later use*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_RESERVED_3 =
      0x2000, /**< Reserved for later use*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_CUSTOM_1 =
      0x4000, /**< Custom free to use flag*/
  XILA_GRAPHICS_BUTTONMATRIX_CTRL_CUSTOM_2 =
      0x8000, /**< Custom free to use flag*/
} Xila_graphics_buttonmatrix_ctrl_t;

typedef enum {
  XILA_GRAPHICS_ARC_MODE_NORMAL,
  XILA_GRAPHICS_ARC_MODE_SYMMETRICAL,
  XILA_GRAPHICS_ARC_MODE_REVERSE
} Xila_graphics_arc_mode_t;

typedef enum {
  XILA_GRAPHICS_MENU_HEADER_TOP_FIXED,   /**< Header is positioned at the top */
  XILA_GRAPHICS_MENU_HEADER_TOP_UNFIXED, /**< Header is positioned at the top
                                            and can be scrolled out of view*/
  XILA_GRAPHICS_MENU_HEADER_BOTTOM_FIXED /**< Header is positioned at the bottom
                                          */
} Xila_graphics_menu_mode_header_t;

typedef enum {
  XILA_GRAPHICS_MENU_ROOT_BACK_BUTTON_DISABLED,
  XILA_GRAPHICS_MENU_ROOT_BACK_BUTTON_ENABLED
} Xila_graphics_menu_mode_root_back_button_t;

/** Roller mode. */
typedef enum {
  XILA_GRAPHICS_ROLLER_MODE_NORMAL, /**< Normal mode (roller ends at the end of
                                       the options). */
  XILA_GRAPHICS_ROLLER_MODE_INFINITE, /**< Infinite mode (roller can be scrolled
                                         forever). */
} Xila_graphics_roller_mode_t;

typedef enum {
  XILA_GRAPHICS_TABLE_CELL_CTRL_MERGE_RIGHT = 1 << 0,
  XILA_GRAPHICS_TABLE_CELL_CTRL_TEXT_CROP = 1 << 1,
  XILA_GRAPHICS_TABLE_CELL_CTRL_CUSTOM_1 = 1 << 4,
  XILA_GRAPHICS_TABLE_CELL_CTRL_CUSTOM_2 = 1 << 5,
  XILA_GRAPHICS_TABLE_CELL_CTRL_CUSTOM_3 = 1 << 6,
  XILA_GRAPHICS_TABLE_CELL_CTRL_CUSTOM_4 = 1 << 7,
} Xila_graphics_table_cell_ctrl_t;

/**
 * On/Off features controlling the object's behavior.
 * OR-ed values are possible
 *
 * Note: update obj flags corresponding properties below
 * whenever add/remove flags or change bit definition of flags.
 */
typedef enum {
  XILA_GRAPHICS_OBJECT_FLAG_HIDDEN =
      (1L << 0), /**< Make the object hidden. (Like it wasn't there at all)*/
  XILA_GRAPHICS_OBJECT_FLAG_CLICKABLE =
      (1L << 1), /**< Make the object clickable by the input devices*/
  XILA_GRAPHICS_OBJECT_FLAG_CLICK_FOCUSABLE =
      (1L << 2), /**< Add focused state to the object when clicked*/
  XILA_GRAPHICS_OBJECT_FLAG_CHECKABLE =
      (1L << 3), /**< Toggle checked state when the object is clicked*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLLABLE =
      (1L << 4), /**< Make the object scrollable*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_ELASTIC =
      (1L << 5), /**< Allow scrolling inside but with slower speed*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_MOMENTUM =
      (1L << 6), /**< Make the object scroll further when "thrown"*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_ONE =
      (1L << 7), /**< Allow scrolling only one snappable children*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_CHAIN_HOR =
      (1L << 8), /**< Allow propagating the horizontal scroll to a parent*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_CHAIN_VER =
      (1L << 9), /**< Allow propagating the vertical scroll to a parent*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_CHAIN =
      (XILA_GRAPHICS_OBJECT_FLAG_SCROLL_CHAIN_HOR |
       XILA_GRAPHICS_OBJECT_FLAG_SCROLL_CHAIN_VER),
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_ON_FOCUS =
      (1L << 10), /**< Automatically scroll object to make it visible when
                     focused*/
  XILA_GRAPHICS_OBJECT_FLAG_SCROLL_WITH_ARROW =
      (1L << 11), /**< Allow scrolling the focused object with arrow keys*/
  XILA_GRAPHICS_OBJECT_FLAG_SNAPPABLE =
      (1L << 12), /**< If scroll snap is enabled on the parent it can snap to
                     this object*/
  XILA_GRAPHICS_OBJECT_FLAG_PRESS_LOCK =
      (1L << 13), /**< Keep the object pressed even if the press slid from the
                     object*/
  XILA_GRAPHICS_OBJECT_FLAG_EVENT_BUBBLE =
      (1L << 14), /**< Propagate the events to the parent too*/
  XILA_GRAPHICS_OBJECT_FLAG_GESTURE_BUBBLE =
      (1L << 15), /**< Propagate the gestures to the parent*/
  XILA_GRAPHICS_OBJECT_FLAG_ADV_HITTEST =
      (1L << 16), /**< Allow performing more accurate hit (click) test. E.g.
                     consider rounded corners.*/
  XILA_GRAPHICS_OBJECT_FLAG_IGNORE_LAYOUT =
      (1L << 17), /**< Make the object not positioned by the layouts*/
  XILA_GRAPHICS_OBJECT_FLAG_FLOATING =
      (1L << 18), /**< Do not scroll the object when the parent scrolls and
                     ignore layout*/
  XILA_GRAPHICS_OBJECT_FLAG_SEND_DRAW_TASK_EVENTS =
      (1L << 19), /**< Send `XILA_GRAPHICS_EVENT_DRAW_TASK_ADDED` events*/
  XILA_GRAPHICS_OBJECT_FLAG_OVERFLOW_VISIBLE =
      (1L << 20), /**< Do not clip the children to the parent's ext draw size*/
  XILA_GRAPHICS_OBJECT_FLAG_FLEX_IN_NEW_TRACK =
      (1L << 21), /**< Start a new flex track on this item*/

  XILA_GRAPHICS_OBJECT_FLAG_LAYOUT_1 =
      (1L << 23), /**< Custom flag, free to use by layouts*/
  XILA_GRAPHICS_OBJECT_FLAG_LAYOUT_2 =
      (1L << 24), /**< Custom flag, free to use by layouts*/

  XILA_GRAPHICS_OBJECT_FLAG_WIDGET_1 =
      (1L << 25), /**< Custom flag, free to use by widget*/
  XILA_GRAPHICS_OBJECT_FLAG_WIDGET_2 =
      (1L << 26), /**< Custom flag, free to use by widget*/
  XILA_GRAPHICS_OBJECT_FLAG_USER_1 =
      (1L << 27), /**< Custom flag, free to use by user*/
  XILA_GRAPHICS_OBJECT_FLAG_USER_2 =
      (1L << 28), /**< Custom flag, free to use by user*/
  XILA_GRAPHICS_OBJECT_FLAG_USER_3 =
      (1L << 29), /**< Custom flag, free to use by user*/
  XILA_GRAPHICS_OBJECT_FLAG_USER_4 =
      (1L << 30), /**< Custom flag, free to use by user*/
} Xila_graphics_object_flag_t;

/**
 * LVGL error codes.
 */
typedef enum {
  XILA_GRAPHICS_RESULT_INVALID =
      0, /*Typically indicates that the object is deleted (become invalid) in
the action function or an operation was failed*/
  XILA_GRAPHICS_RESULT_OK, /*The object is valid (no deleted) after the action*/
} Xila_graphics_result_t;

typedef uint16_t Xila_graphics_state_t;

enum {
  XILA_GRAPHICS_STATE_DEFAULT = 0x0000,
  XILA_GRAPHICS_STATE_CHECKED = 0x0001,
  XILA_GRAPHICS_STATE_FOCUSED = 0x0002,
  XILA_GRAPHICS_STATE_FOCUS_KEY = 0x0004,
  XILA_GRAPHICS_STATE_EDITED = 0x0008,
  XILA_GRAPHICS_STATE_HOVERED = 0x0010,
  XILA_GRAPHICS_STATE_PRESSED = 0x0020,
  XILA_GRAPHICS_STATE_SCROLLED = 0x0040,
  XILA_GRAPHICS_STATE_DISABLED = 0x0080,
  XILA_GRAPHICS_STATE_USER_1 = 0x1000,
  XILA_GRAPHICS_STATE_USER_2 = 0x2000,
  XILA_GRAPHICS_STATE_USER_3 = 0x4000,
  XILA_GRAPHICS_STATE_USER_4 = 0x8000,

  XILA_GRAPHICS_STATE_ANY = 0xFFFF, /**< Special value can be used in some
                                       functions to target all states*/
};

enum {
  XILA_GRAPHICS_PART_MAIN = 0x000000,      /**< A background like rectangle*/
  XILA_GRAPHICS_PART_SCROLLBAR = 0x010000, /**< The scrollbar(s)*/
  XILA_GRAPHICS_PART_INDICATOR =
      0x020000, /**< Indicator, e.g. for slider, bar, switch, or the tick box of
                   the checkbox*/
  XILA_GRAPHICS_PART_KNOB =
      0x030000, /**< Like handle to grab to adjust the value*/
  XILA_GRAPHICS_PART_SELECTED =
      0x040000, /**< Indicate the currently selected option or section*/
  XILA_GRAPHICS_PART_ITEMS = 0x050000,  /**< Used if the widget has multiple
                                           similar elements (e.g. table cells)*/
  XILA_GRAPHICS_PART_CURSOR = 0x060000, /**< Mark a specific place e.g. for text
                                           area's cursor or on a chart*/

  XILA_GRAPHICS_PART_CUSTOM_FIRST =
      0x080000, /**< Extension point for custom widgets*/

  XILA_GRAPHICS_PART_ANY = 0x0F0000, /**< Special value can be used in some
                                        functions to target all parts*/
};

/**
 * Options for text rendering.
 */

typedef enum {
  XILA_GRAPHICS_TEXT_FLAG_NONE = 0x00,
  XILA_GRAPHICS_TEXT_FLAG_EXPAND =
      0x01, /**< Ignore max-width to avoid automatic word wrapping*/
  XILA_GRAPHICS_TEXT_FLAG_FIT =
      0x02, /**< Max-width is already equal to the longest line. (Used to skip
               some calculation)*/
  XILA_GRAPHICS_TEXT_FLAG_BREAK_ALL =
      0x04, /**< To prevent overflow, insert breaks between any two characters.
      Otherwise breaks are inserted at word boundaries, as configured via
      XILA_GRAPHICS_TXT_BREAK_CHARS or according to
      XILA_GRAPHICS_TXT_LINE_BREAK_LONG_LEN,
      XILA_GRAPHICS_TXT_LINE_BREAK_LONG_PRE_MIN_LEN, and
      XILA_GRAPHICS_TXT_LINE_BREAK_LONG_POST_MIN_LEN.*/
} Xila_graphics_text_flag_t;

/** Label align policy*/
typedef enum {
  XILA_GRAPHICS_TEXT_ALIGN_AUTO,   /**< Align text auto*/
  XILA_GRAPHICS_TEXT_ALIGN_LEFT,   /**< Align text to left*/
  XILA_GRAPHICS_TEXT_ALIGN_CENTER, /**< Align text to center*/
  XILA_GRAPHICS_TEXT_ALIGN_RIGHT,  /**< Align text to right*/
} Xila_graphics_text_align_t;

typedef enum {
  /** No flags */
  XILA_GRAPHICS_OBJECT_POINT_TRANSFORM_FLAG_NONE = 0x00,

  /** Consider the transformation properties of the parents too */
  XILA_GRAPHICS_OBJECT_POINT_TRANSFORM_FLAG_RECURSIVE = 0x01,

  /** Execute the inverse of the transformation (-angle and 1/zoom) */
  XILA_GRAPHICS_OBJECT_POINT_TRANSFORM_FLAG_INVERSE = 0x02,

  /** Both inverse and recursive*/
  XILA_GRAPHICS_OBJECT_POINT_TRANSFORM_FLAG_INVERSE_RECURSIVE = 0x03,
} Xila_graphics_object_point_transform_flag_t;

typedef struct {
} Xila_graphics_group_t;

/** Can be used to indicate if animations are enabled or disabled in a case*/
typedef enum {
  XILA_GRAPHICS_ANIM_OFF,
  XILA_GRAPHICS_ANIM_ON,
} Xila_graphics_anim_enable_t;

typedef enum {
  XILA_GRAPHICS_BASE_DIR_LTR = 0x00,
  XILA_GRAPHICS_BASE_DIR_RTL = 0x01,
  XILA_GRAPHICS_BASE_DIR_AUTO = 0x02,

  XILA_GRAPHICS_BASE_DIR_NEUTRAL = 0x20,
  XILA_GRAPHICS_BASE_DIR_WEAK = 0x21,
} Xila_graphics_base_dir_t;

typedef struct {
} Xila_graphics_display_t;

/** Scrollbar modes: shows when should the scrollbars be visible*/
typedef enum {
  XILA_GRAPHICS_SCROLLBAR_MODE_OFF,    /**< Never show scrollbars*/
  XILA_GRAPHICS_SCROLLBAR_MODE_ON,     /**< Always show scrollbars*/
  XILA_GRAPHICS_SCROLLBAR_MODE_ACTIVE, /**< Show scroll bars when object is
                                          being scrolled*/
  XILA_GRAPHICS_SCROLLBAR_MODE_AUTO,   /**< Show scroll bars when the content is
                                          large enough to be scrolled*/
} Xila_graphics_scrollbar_mode_t;

/** Scroll span align options. Tells where to align the snappable children when
 * scroll stops.*/
typedef enum {
  XILA_GRAPHICS_SCROLL_SNAP_NONE,  /**< Do not align, leave where it is*/
  XILA_GRAPHICS_SCROLL_SNAP_START, /**< Align to the left/top*/
  XILA_GRAPHICS_SCROLL_SNAP_END,   /**< Align to the right/bottom*/
  XILA_GRAPHICS_SCROLL_SNAP_CENTER /**< Align to the center*/
} Xila_graphics_scroll_snap_t;

/**
 * Scale mode
 */
typedef enum {
  XILA_GRAPHICS_SCALE_MODE_HORIZONTAL_TOP = 0x00U,
  XILA_GRAPHICS_SCALE_MODE_HORIZONTAL_BOTTOM = 0x01U,
  XILA_GRAPHICS_SCALE_MODE_VERTICAL_LEFT = 0x02U,
  XILA_GRAPHICS_SCALE_MODE_VERTICAL_RIGHT = 0x04U,
  XILA_GRAPHICS_SCALE_MODE_ROUND_INNER = 0x08U,
  XILA_GRAPHICS_SCALE_MODE_ROUND_OUTER = 0x10U,
  XILA_GRAPHICS_SCALE_MODE_LAST
} Xila_graphics_scale_mode_t;

typedef struct {
} Xila_graphics_scale_section_t;

typedef enum {
  XILA_GRAPHICS_BAR_MODE_NORMAL,
  XILA_GRAPHICS_BAR_MODE_SYMMETRICAL,
  XILA_GRAPHICS_BAR_MODE_RANGE
} Xila_graphics_bar_mode_t;

typedef enum {
  XILA_GRAPHICS_SLIDER_MODE_NORMAL = XILA_GRAPHICS_BAR_MODE_NORMAL,
  XILA_GRAPHICS_SLIDER_MODE_SYMMETRICAL = XILA_GRAPHICS_BAR_MODE_SYMMETRICAL,
  XILA_GRAPHICS_SLIDER_MODE_RANGE = XILA_GRAPHICS_BAR_MODE_RANGE
} Xila_graphics_slider_mode_t;

typedef enum {
  XILA_GRAPHICS_BAR_ORIENTATION_AUTO,
  XILA_GRAPHICS_BAR_ORIENTATION_HORIZONTAL,
  XILA_GRAPHICS_BAR_ORIENTATION_VERTICAL
} Xila_graphics_bar_orientation_t;

typedef enum {
  XILA_GRAPHICS_SPAN_OVERFLOW_CLIP,
  XILA_GRAPHICS_SPAN_OVERFLOW_ELLIPSIS,
  XILA_GRAPHICS_SPAN_OVERFLOW_LAST, /**< Fence member*/
} Xila_graphics_span_overflow_t;

typedef enum {
  XILA_GRAPHICS_SPAN_MODE_FIXED,  /**< fixed the obj size */
  XILA_GRAPHICS_SPAN_MODE_EXPAND, /**< Expand the object size to the text size
                                   */
  XILA_GRAPHICS_SPAN_MODE_BREAK,  /**< Keep width, break the too long lines and
                                     expand height */
  XILA_GRAPHICS_SPAN_MODE_LAST    /**< Fence member */
} Xila_graphics_span_mode_t;

typedef struct {
} Xila_graphics_chart_cursor_t;

/**
 * A common type to handle all the property types in the same way.
 */
typedef union {
  int32_t num; /**< Number integer number (opacity, enums, booleans or "normal"
                  numbers)*/
  const void *ptr;             /**< Constant pointers  (font, cone text, etc)*/
  Xila_graphics_color_t color; /**< Colors*/
} Xila_graphics_style_value_t;

typedef struct {
} Xila_graphics_layer_t;

/**
 * Chart types
 */
typedef enum {
  LV_CHART_TYPE_NONE,    /**< Don't draw the series*/
  LV_CHART_TYPE_LINE,    /**< Connect the points with lines*/
  LV_CHART_TYPE_BAR,     /**< Draw columns*/
  LV_CHART_TYPE_SCATTER, /**< Draw points and lines in 2D (x,y coordinates)*/
} Xila_graphics_chart_type_t;

/**
 * Chart update mode for `lv_chart_set_next`
 */
typedef enum {
  LV_CHART_UPDATE_MODE_SHIFT,    /**< Shift old data to the left and add the new
                                    one the right*/
  LV_CHART_UPDATE_MODE_CIRCULAR, /**< Add the new data in a circular way*/
} Xila_graphics_chart_update_mode_t;

/**
 * Enumeration of the axis'
 */
typedef enum {
  LV_CHART_AXIS_PRIMARY_Y = 0x00,
  LV_CHART_AXIS_SECONDARY_Y = 0x01,
  LV_CHART_AXIS_PRIMARY_X = 0x02,
  LV_CHART_AXIS_SECONDARY_X = 0x04,
  LV_CHART_AXIS_LAST
} Xila_graphics_chart_axis_t;

typedef enum {
  LV_STYLE_RES_NOT_FOUND,
  LV_STYLE_RES_FOUND,
} Xila_graphics_style_res_t;

typedef enum {
  LV_EVENT_ALL = 0,

  /** Input device events*/
  LV_EVENT_PRESSED,  /**< The object has been pressed*/
  LV_EVENT_PRESSING, /**< The object is being pressed (called continuously while
                        pressing)*/
  LV_EVENT_PRESS_LOST,    /**< The object is still being pressed but slid
                             cursor/finger off of the object */
  LV_EVENT_SHORT_CLICKED, /**< The object was pressed for a short period of
                             time, then released it. Not called if scrolled.*/
  LV_EVENT_LONG_PRESSED,  /**< Object has been pressed for at least
                             `long_press_time`.  Not called if scrolled.*/
  LV_EVENT_LONG_PRESSED_REPEAT, /**< Called after `long_press_time` in every
                                   `long_press_repeat_time` ms.  Not called if
                                   scrolled.*/
  LV_EVENT_CLICKED,  /**< Called on release if not scrolled (regardless to long
                        press)*/
  LV_EVENT_RELEASED, /**< Called in every cases when the object has been
                        released*/
  LV_EVENT_SCROLL_BEGIN, /**< Scrolling begins. The event parameter is a pointer
                            to the animation of the scroll. Can be modified*/
  LV_EVENT_SCROLL_THROW_BEGIN,
  LV_EVENT_SCROLL_END, /**< Scrolling ends*/
  LV_EVENT_SCROLL,     /**< Scrolling*/
  LV_EVENT_GESTURE,    /**< A gesture is detected. Get the gesture with
                          `lv_indev_get_gesture_dir(lv_indev_active());` */
  LV_EVENT_KEY,        /**< A key is sent to the object. Get the key with
                          `lv_indev_get_key(lv_indev_active());`*/
  LV_EVENT_ROTARY,  /**< An encoder or wheel was rotated. Get the rotation count
                       with `lv_event_get_rotary_diff(e);`*/
  LV_EVENT_FOCUSED, /**< The object is focused*/
  LV_EVENT_DEFOCUSED,   /**< The object is defocused*/
  LV_EVENT_LEAVE,       /**< The object is defocused but still selected*/
  LV_EVENT_HIT_TEST,    /**< Perform advanced hit-testing*/
  LV_EVENT_INDEV_RESET, /**< Indev has been reset*/
  LV_EVENT_HOVER_OVER,  /**< Indev hover over object*/
  LV_EVENT_HOVER_LEAVE, /**< Indev hover leave object*/

  /** Drawing events*/
  LV_EVENT_COVER_CHECK, /**< Check if the object fully covers an area. The event
                           parameter is `lv_cover_check_info_t *`.*/
  LV_EVENT_REFR_EXT_DRAW_SIZE, /**< Get the required extra draw area around the
                                  object (e.g. for shadow). The event parameter
                                  is `int32_t *` to store the size.*/
  LV_EVENT_DRAW_MAIN_BEGIN,    /**< Starting the main drawing phase*/
  LV_EVENT_DRAW_MAIN,          /**< Perform the main drawing*/
  LV_EVENT_DRAW_MAIN_END,      /**< Finishing the main drawing phase*/
  LV_EVENT_DRAW_POST_BEGIN, /**< Starting the post draw phase (when all children
                               are drawn)*/
  LV_EVENT_DRAW_POST, /**< Perform the post draw phase (when all children are
                         drawn)*/
  LV_EVENT_DRAW_POST_END, /**< Finishing the post draw phase (when all children
                             are drawn)*/
  LV_EVENT_DRAW_TASK_ADDED, /**< Adding a draw task */

  /** Special events*/
  LV_EVENT_VALUE_CHANGED, /**< The object's value has changed (i.e. slider
                             moved)*/
  LV_EVENT_INSERT,  /**< A text is inserted to the object. The event data is
                       `char *` being inserted.*/
  LV_EVENT_REFRESH, /**< Notify the object to refresh something on it (for the
                       user)*/
  LV_EVENT_READY,   /**< A process has finished*/
  LV_EVENT_CANCEL,  /**< A process has been cancelled */

  /** Other events*/
  LV_EVENT_CREATE,        /**< Object is being created*/
  LV_EVENT_DELETE,        /**< Object is being deleted*/
  LV_EVENT_CHILD_CHANGED, /**< Child was removed, added, or its size, position
                             were changed */
  LV_EVENT_CHILD_CREATED, /**< Child was created, always bubbles up to all
                             parents*/
  LV_EVENT_CHILD_DELETED, /**< Child was deleted, always bubbles up to all
                             parents*/
  LV_EVENT_SCREEN_UNLOAD_START, /**< A screen unload started, fired immediately
                                   when scr_load is called*/
  LV_EVENT_SCREEN_LOAD_START, /**< A screen load started, fired when the screen
                                 change delay is expired*/
  LV_EVENT_SCREEN_LOADED,     /**< A screen was loaded*/
  LV_EVENT_SCREEN_UNLOADED,   /**< A screen was unloaded*/
  LV_EVENT_SIZE_CHANGED,      /**< Object coordinates/size have changed*/
  LV_EVENT_STYLE_CHANGED,     /**< Object's style has changed*/
  LV_EVENT_LAYOUT_CHANGED,    /**< The children position has changed due to a
                                 layout recalculation*/
  LV_EVENT_GET_SELF_SIZE,     /**< Get the internal size of a widget*/

  /** Events of optional LVGL components*/
  LV_EVENT_INVALIDATE_AREA,
  LV_EVENT_RESOLUTION_CHANGED,
  LV_EVENT_COLOR_FORMAT_CHANGED,
  LV_EVENT_REFR_REQUEST,
  LV_EVENT_REFR_START,
  LV_EVENT_REFR_READY,
  LV_EVENT_RENDER_START,
  LV_EVENT_RENDER_READY,
  LV_EVENT_FLUSH_START,
  LV_EVENT_FLUSH_FINISH,
  LV_EVENT_FLUSH_WAIT_START,
  LV_EVENT_FLUSH_WAIT_FINISH,

  LV_EVENT_VSYNC,

  LV_EVENT_LAST, /** Number of default events*/

  LV_EVENT_PREPROCESS =
      0x8000, /** This is a flag that can be set with an event so it's processed
                before the class default event processing */
} Xila_graphics_event_code_t;

typedef enum {
  LV_LABEL_LONG_WRAP, /**< Keep the object width, wrap lines longer than object
                         width and expand the object height*/
  LV_LABEL_LONG_DOT, /**< Keep the size and write dots at the end if the text is
                        too long*/
  LV_LABEL_LONG_SCROLL, /**< Keep the size and roll the text back and forth*/
  LV_LABEL_LONG_SCROLL_CIRCULAR, /**< Keep the size and roll the text
                                    circularly*/
  LV_LABEL_LONG_CLIP, /**< Keep the size and clip the text out of it*/
} Xila_graphics_label_long_mode_t;

typedef enum {
  LV_SCR_LOAD_ANIM_NONE,
  LV_SCR_LOAD_ANIM_OVER_LEFT,
  LV_SCR_LOAD_ANIM_OVER_RIGHT,
  LV_SCR_LOAD_ANIM_OVER_TOP,
  LV_SCR_LOAD_ANIM_OVER_BOTTOM,
  LV_SCR_LOAD_ANIM_MOVE_LEFT,
  LV_SCR_LOAD_ANIM_MOVE_RIGHT,
  LV_SCR_LOAD_ANIM_MOVE_TOP,
  LV_SCR_LOAD_ANIM_MOVE_BOTTOM,
  LV_SCR_LOAD_ANIM_FADE_IN,
  LV_SCR_LOAD_ANIM_FADE_ON =
      LV_SCR_LOAD_ANIM_FADE_IN, /*For backward compatibility*/
  LV_SCR_LOAD_ANIM_FADE_OUT,
  LV_SCR_LOAD_ANIM_OUT_LEFT,
  LV_SCR_LOAD_ANIM_OUT_RIGHT,
  LV_SCR_LOAD_ANIM_OUT_TOP,
  LV_SCR_LOAD_ANIM_OUT_BOTTOM,
} Xila_graphics_screen_load_anim_t;

typedef enum {
  LV_SLIDER_ORIENTATION_AUTO,
  LV_SLIDER_ORIENTATION_HORIZONTAL,
  LV_SLIDER_ORIENTATION_VERTICAL
} Xila_graphics_slider_orientation_t;

typedef struct {
} Xila_graphics_matrix_t;