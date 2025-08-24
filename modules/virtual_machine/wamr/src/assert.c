// Custom assert implementation for Xila
extern void Xila_assert(int condition, const char* message);

#ifdef NDEBUG
#define assert(condition) ((void)0)
#else
#define assert(condition) \
    do { \
        if (!(condition)) { \
            Xila_assert(0, "Assertion failed: " #condition " at " __FILE__ ":" __LINE__); \
        } \
    } while(0)
#endif

