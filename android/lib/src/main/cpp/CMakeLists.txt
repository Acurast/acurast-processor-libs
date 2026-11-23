# JNI glue of the benchmark (com.acurast.bench.Acubench). The benchmark itself is in the native
# library of this workspace (libacurast_processor_ffi.so), which the glue links dynamically, so the
# app carries one copy of the Rust code.
cmake_minimum_required(VERSION 3.22.1)

project(acubench)

set(CMAKE_CXX_STANDARD 17)

# Header of the benchmark's C functions.
set(BENCHMARK_FFI_INCLUDE ${CMAKE_SOURCE_DIR}/../../../../../benchmark/rust/ffi/include)

add_library(acurast_processor_ffi SHARED IMPORTED)
set_target_properties(acurast_processor_ffi PROPERTIES
        IMPORTED_LOCATION ${RUST_JNI_LIBS}/${ANDROID_ABI}/libacurast_processor_ffi.so)

add_library(acubench SHARED
        src/acubench.h
        src/acubench.cpp)
target_include_directories(acubench PRIVATE ${BENCHMARK_FFI_INCLUDE})
target_link_libraries(acubench acurast_processor_ffi android log)

add_library(acubenchtest SHARED
        test/ffi_test.h
        test/ffi_test.cpp)
target_include_directories(acubenchtest PRIVATE ${BENCHMARK_FFI_INCLUDE})
target_link_libraries(acubenchtest acubench android log)
