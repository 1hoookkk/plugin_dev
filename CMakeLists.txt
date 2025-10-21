cmake_minimum_required(VERSION 3.24)

# Set project metadata
project(EngineField
    VERSION 1.0.0
    DESCRIPTION "Audio Plugin using JUCE framework (authentic EMU Z-plane)"
    LANGUAGES C CXX)

# ============================================================================
# GLOBAL C++ CONFIGURATION
# ============================================================================
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Position-independent code for all platforms (required for plugins)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Export compile commands for IDE integration
if(NOT CMAKE_GENERATOR MATCHES "Visual Studio" AND NOT CMAKE_GENERATOR MATCHES "Xcode")
    set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
endif()

# ============================================================================
# PLATFORM-SPECIFIC RUNTIME CONFIGURATION
# ============================================================================

# Windows: static MSVC runtime (CRT)
if(MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
    # Enable standards-conformant mode and C++ standard reporting
    add_compile_options(/permissive- /Zc:__cplusplus)
endif()

# macOS: deployment target for binary compatibility
if(APPLE)
    set(CMAKE_OSX_DEPLOYMENT_TARGET "10.13" CACHE STRING "macOS deployment target")
    message(STATUS "macOS deployment target: ${CMAKE_OSX_DEPLOYMENT_TARGET}")
endif()

# ============================================================================
# COMPILER FLAGS BY BUILD TYPE
# ============================================================================

# Compiler detection and flags
if(MSVC)
    # MSVC (Visual Studio 2022)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /W4 /fp:fast")
    set(CMAKE_CXX_FLAGS_RELEASE "/Ox /Ob2 /Oi /Ot /GL /DNDEBUG /MD")
    set(CMAKE_CXX_FLAGS_DEBUG "/Zi /Od /RTC1 /D_DEBUG /MDd")
    set(CMAKE_SHARED_LINKER_FLAGS_RELEASE "${CMAKE_SHARED_LINKER_FLAGS_RELEASE} /LTCG")

else()
    # Clang / GCC (macOS, Linux)
    # Common flags for both compilers
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -ffast-math -fno-finite-math-only")

    # Release build: aggressive optimization + LTO
    set(CMAKE_CXX_FLAGS_RELEASE "-O3 -flto -DNDEBUG")
    set(CMAKE_SHARED_LINKER_FLAGS_RELEASE "${CMAKE_SHARED_LINKER_FLAGS_RELEASE} -flto")

    # Debug build: symbols + no optimization + optional sanitizers
    set(CMAKE_CXX_FLAGS_DEBUG "-g -O0 -D_DEBUG")

    # Platform-specific optimizations
    if(APPLE)
        # macOS: additional Clang optimizations
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fvectorize -fslp-vectorize")
    elseif(LINUX OR CMAKE_SYSTEM_NAME STREQUAL "Linux")
        # Linux: enable additional GCC optimizations if available
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -march=x86-64")
    endif()
endif()

# ============================================================================
# WARNING CONFIGURATION (All Platforms)
# ============================================================================

# Default: warnings ON, errors OFF (developers can enable via preset)
option(TREAT_WARNINGS_AS_ERRORS "Convert compiler warnings to errors" OFF)

if(MSVC)
    # MSVC: W4 = -Wall -Wextra equivalent
    add_compile_options(/W4)

    if(TREAT_WARNINGS_AS_ERRORS)
        add_compile_options(/WX)
    endif()

    # Suppress false positives from JUCE
    add_compile_options(
        /wd4458  # declaration hides class member
        /wd4275  # non-DLL-interface class used as base for DLL-interface class
        /wd4996  # deprecation
    )
else()
    # GCC / Clang: comprehensive warning set
    add_compile_options(
        -Wall
        -Wextra
        -Wpedantic
        -Wconversion
        -Wshadow
        -Wnon-virtual-dtor
        -Wold-style-cast
        -Wcast-align
        -Woverloaded-virtual
    )

    if(TREAT_WARNINGS_AS_ERRORS)
        add_compile_options(-Werror)
    endif()

    # Suppress false positives from JUCE / third-party code
    add_compile_options(
        -Wno-c++98-compat
        -Wno-unknown-pragmas
        -Wno-deprecated-declarations
    )
endif()

# ============================================================================
# OPTIONAL SANITIZERS (Debug builds only)
# ============================================================================

option(ENABLE_ASAN "Enable Address Sanitizer in Debug builds" OFF)

if(ENABLE_ASAN AND CMAKE_BUILD_TYPE STREQUAL "Debug" AND NOT MSVC)
    add_compile_options(-fsanitize=address -fno-omit-frame-pointer)
    add_link_options(-fsanitize=address)
    message(STATUS "Address Sanitizer enabled (Debug only)")
endif()

# ============================================================================
# JUCE DISCOVERY
# ============================================================================

# Allow JUCE to be discovered via environment variables or cache variables
if(DEFINED ENV{JUCE_ROOT})
    list(APPEND CMAKE_PREFIX_PATH "$ENV{JUCE_ROOT}")
    message(STATUS "JUCE_ROOT from environment: $ENV{JUCE_ROOT}")
endif()

if(DEFINED JUCE_SOURCE_DIR)
    list(APPEND CMAKE_PREFIX_PATH "${JUCE_SOURCE_DIR}")
    message(STATUS "JUCE_SOURCE_DIR from cache: ${JUCE_SOURCE_DIR}")
endif()

find_package(JUCE CONFIG REQUIRED)
message(STATUS "JUCE ${JUCE_VERSION} found at: ${JUCE_DIR}")

# Resolve juceaide executable (required for plugin generation)
if(TARGET juce::juceaide)
    get_target_property(_juceaide_loc juce::juceaide IMPORTED_LOCATION)
    set(_juceaide_candidate "${_juceaide_loc}")

    if(NOT _juceaide_candidate OR IS_DIRECTORY "${_juceaide_candidate}")
        get_filename_component(_juce_prefix "${JUCE_DIR}/../../.." ABSOLUTE)
        set(_juceaide_candidate "${_juce_prefix}/bin/JUCE-${JUCE_VERSION}/juceaide${CMAKE_EXECUTABLE_SUFFIX}")
    endif()

    if(EXISTS "${_juceaide_candidate}")
        set_target_properties(juce::juceaide PROPERTIES IMPORTED_LOCATION "${_juceaide_candidate}")
        if(NOT TARGET juceaide)
            add_executable(juceaide IMPORTED GLOBAL)
            set_target_properties(juceaide PROPERTIES IMPORTED_LOCATION "${_juceaide_candidate}")
        endif()
        message(STATUS "juceaide resolved to: ${_juceaide_candidate}")
    else()
        message(WARNING "juceaide not found at expected path: ${_juceaide_candidate}")
    endif()
endif()

# ============================================================================
# BUILD OPTIONS
# ============================================================================

option(BUILD_STANDALONE "Build standalone version of the plugin" ON)
option(BUILD_VST3 "Build VST3 version of the plugin" ON)

# pluginval path: can be specified as cache variable or environment variable
if(NOT DEFINED PLUGINVAL_EXE)
    if(DEFINED ENV{PLUGINVAL_EXE})
        set(PLUGINVAL_EXE "$ENV{PLUGINVAL_EXE}" CACHE FILEPATH "Path to pluginval executable")
    else()
        set(PLUGINVAL_EXE "" CACHE FILEPATH "Path to pluginval executable (optional for CI validation)")
    endif()
endif()

message(STATUS "Build configuration:")
message(STATUS "  - Standalone: ${BUILD_STANDALONE}")
message(STATUS "  - VST3: ${BUILD_VST3}")
message(STATUS "  - pluginval: ${PLUGINVAL_EXE}")
message(STATUS "  - Warnings as errors: ${TREAT_WARNINGS_AS_ERRORS}")

# ============================================================================
# INCLUDE PLUGIN TARGETS
# ============================================================================

add_subdirectory(plugins/EngineField)

# ============================================================================
# OPTIONAL: UI PROBE (Manual diagnostics tool)
# ============================================================================

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tools/ui_probe.cpp")
    add_executable(ui_probe EXCLUDE_FROM_ALL tools/ui_probe.cpp)
    target_link_libraries(ui_probe PRIVATE
        EngineField_SharedCode
        juce::juce_audio_basics
        juce::juce_audio_processors
        juce::juce_core
    )
endif()

# ============================================================================
# PLUGINVAL VALIDATION TARGET
# ============================================================================

# Create pluginval target for VST3 plugin validation (strict 5-level check)
if(BUILD_VST3 AND PLUGINVAL_EXE)
    if(EXISTS "${PLUGINVAL_EXE}")
        # Determine output directory based on generator
        if(CMAKE_GENERATOR MATCHES "Visual Studio")
            set(_vst3_output_dir "${CMAKE_BINARY_DIR}/plugins/EngineField/EngineField_artefacts/$<CONFIG>/VST3")
        else()
            set(_vst3_output_dir "${CMAKE_BINARY_DIR}/plugins/EngineField/EngineField_artefacts/VST3")
        endif()

        add_custom_target(pluginval_vst3
            COMMAND "${PLUGINVAL_EXE}"
                --strictness-level 5
                --validate
                "${_vst3_output_dir}/EngineField.vst3"
                --timeout-ms 120000
            COMMENT "Running pluginval on EngineField.vst3 (strictness=5)"
            DEPENDS EngineField_VST3
            VERBATIM
        )
        message(STATUS "pluginval target 'pluginval_vst3' added (will validate ${_vst3_output_dir}/EngineField.vst3)")
    else()
        message(WARNING "pluginval not found at: ${PLUGINVAL_EXE}")
    endif()
else()
    if(NOT BUILD_VST3)
        message(STATUS "pluginval target skipped (VST3 build disabled)")
    else()
        message(STATUS "pluginval target skipped (PLUGINVAL_EXE not defined)")
    endif()
endif()

# ============================================================================
# SUMMARY MESSAGE
# ============================================================================

message(STATUS "")
message(STATUS "Engine:Field Build Configuration Summary:")
message(STATUS "  CMAKE_BUILD_TYPE: ${CMAKE_BUILD_TYPE}")
message(STATUS "  CMAKE_CXX_COMPILER: ${CMAKE_CXX_COMPILER}")
message(STATUS "  CMAKE_CXX_FLAGS: ${CMAKE_CXX_FLAGS}")
message(STATUS "  CMAKE_CXX_FLAGS_RELEASE: ${CMAKE_CXX_FLAGS_RELEASE}")
message(STATUS "  CMAKE_CXX_FLAGS_DEBUG: ${CMAKE_CXX_FLAGS_DEBUG}")
if(APPLE)
    message(STATUS "  macOS deployment target: ${CMAKE_OSX_DEPLOYMENT_TARGET}")
endif()
message(STATUS "")
message(STATUS "To build, run:")
message(STATUS "  cmake --preset <preset-name>")
message(STATUS "  cmake --build --preset <build-preset>")
message(STATUS "")
message(STATUS "Available presets:")
message(STATUS "  - macOS: macos-debug, macos-release")
message(STATUS "  - Windows: windows-debug, windows-release, windows-clang-debug, windows-clang-release")
message(STATUS "  - Linux: linux-debug, linux-release")
message(STATUS "")
