set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR wasm32)

set(CMAKE_C_COMPILER clang)
set(CMAKE_CXX_COMPILER clang++)

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
set(CMAKE_C_COMPILER_WORKS 1)
set(CMAKE_CXX_COMPILER_WORKS 1)

set(CMAKE_C_FLAGS "--target=wasm32-unknown-unknown" CACHE STRING "" FORCE)
set(CMAKE_CXX_FLAGS "--target=wasm32-unknown-unknown" CACHE STRING "" FORCE)

# Use WAMR's built-in standalone C library headers instead of host paths
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -I${CMAKE_SOURCE_DIR}/core/deps/libc-builtin" CACHE STRING "" FORCE)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -I${CMAKE_SOURCE_DIR}/core/deps/libc-builtin" CACHE STRING "" FORCE)

# =====================================================================
# FORCE PURE INTERPRETER MODE (Bypass AOT/Hardware Requirements)
# =====================================================================

# Disable compilation backends that look for native CPU architectures
set(WAMR_BUILD_AOT 0 CACHE INTERNAL "Force disable AOT" FORCE)
set(WAMR_BUILD_JIT 0 CACHE INTERNAL "Force disable JIT" FORCE)

# Enable the lightweight, portable bytecode interpreter
set(WAMR_BUILD_INTERP 1 CACHE INTERNAL "Force enable Interpreter" FORCE)
set(WAMR_BUILD_FAST_INTERP 1 CACHE INTERNAL "Use Fast Interpreter" FORCE)

# Strip out unnecessary features to minimize browser footprint
set(WAMR_BUILD_LIBC_WASI 0 CACHE INTERNAL "Disable WASI standard imports" FORCE)
set(WAMR_BUILD_MULTI_MODULE 0 CACHE INTERNAL "Disable multi-module" FORCE)
set(WAMR_BUILD_LIB_PTHREAD 0 CACHE INTERNAL "Disable native threads" FORCE)
