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

add_custom_target(wamr_platform_header_policy_check
  COMMAND ${CMAKE_COMMAND} -E env bash ${CMAKE_CURRENT_LIST_DIR}/tools/check-forbidden-includes.sh
  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}
  COMMENT "Checking forbidden host OS headers in wamr platform files")
