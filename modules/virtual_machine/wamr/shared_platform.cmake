set(CMAKE_C_COMPILER_WORKS TRUE)
set(CMAKE_CXX_COMPILER_WORKS TRUE)
set(CMAKE_C_COMPILER_FORCED TRUE)
set(CMAKE_CXX_COMPILER_FORCED TRUE)

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
    set(WAMR_BUILD_WAMR_COMPILER 0)

set(CMAKE_SYSTEM_PROCESSOR wasm32)
set(CMAKE_SYSTEM_NAME Generic)

# Add the platform specific definitions
add_definitions(-DBH_PLATFORM_XILA)

# Add the platform specific include directories
include_directories(${CMAKE_CURRENT_LIST_DIR}/include)

# Add the platform specific source files
include_directories(${WAMR_ROOT_DIR}/core/shared/platform/include)

# Add xila specific source files
file (GLOB_RECURSE source_all ${CMAKE_CURRENT_LIST_DIR}/src/*.c)

# Add the libc-util source files (errno)
include(${WAMR_ROOT_DIR}/core/shared/platform/common/libc-util/platform_common_libc_util.cmake)
set (source_all ${source_all} ${PLATFORM_COMMON_LIBC_UTIL_SOURCE})

set (PLATFORM_SHARED_SOURCE ${source_all} ${PLATFORM_COMMON_MATH_SOURCE})
