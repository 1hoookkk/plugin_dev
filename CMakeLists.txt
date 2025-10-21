cmake_minimum_required(VERSION 3.24)

# Set project metadata
project(EngineField 
    VERSION 1.0.0 
    DESCRIPTION "Audio Plugin using JUCE framework"
    LANGUAGES C CXX)

# Global configuration
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Only export compile commands for single-config generators (Ninja, Makefile, etc.)
if(NOT CMAKE_GENERATOR MATCHES "Visual Studio" AND NOT CMAKE_GENERATOR MATCHES "Xcode")
    set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
endif()

# Allow JUCE to be discovered via JUCE_ROOT or JUCE_SOURCE_DIR hints
if(DEFINED ENV{JUCE_ROOT})
    list(APPEND CMAKE_PREFIX_PATH "$ENV{JUCE_ROOT}")
endif()

if(DEFINED JUCE_SOURCE_DIR)
    list(APPEND CMAKE_PREFIX_PATH "${JUCE_SOURCE_DIR}")
endif()

find_package(JUCE CONFIG REQUIRED)

message(STATUS "JUCE ${JUCE_VERSION} located via ${JUCE_DIR}")

# Ensure juceaide executable resolves to a concrete path when using JUCE packages
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
        message(STATUS "juceaide executable set to ${_juceaide_candidate}")
    else()
        message(WARNING "juceaide executable not found at ${_juceaide_candidate}; custom command generation may fail")
    endif()
endif()

if(MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
endif()

# Build options with clear descriptions
option(BUILD_STANDALONE "Build standalone version of the plugin" ON)
option(BUILD_VST3 "Build VST3 version of the plugin" ON)
option(ENABLE_PLUGINVAL "Enable automatic plugin validation with pluginval" OFF)

# Set warning levels
if(MSVC)
    add_compile_options(/W4 /permissive- /Zc:__cplusplus)
else()
    add_compile_options(
        -Wall 
        -Wextra 
        -Wconversion 
        -Wshadow 
        -Wnon-virtual-dtor 
        -Wold-style-cast 
        -Wcast-align 
        -Wunused 
        -Woverloaded-virtual 
        -Wpedantic 
        -Wno-c++98-compat 
        -Wno-unknown-pragmas
    )
endif()

# Add the plugin
add_subdirectory(plugins/EngineField)

# Offline probe to exercise FieldProcessor without a host (manual trigger via build target)
add_executable(ui_probe EXCLUDE_FROM_ALL tools/ui_probe.cpp)
target_link_libraries(ui_probe PRIVATE
    EngineField_SharedCode
    juce::juce_audio_basics
    juce::juce_audio_processors
    juce::juce_core
)

# Pluginval integration
if(BUILD_VST3 AND DEFINED PLUGINVAL_EXE)
    if(EXISTS "${PLUGINVAL_EXE}")
        add_custom_target(pluginval_field
            COMMAND "${PLUGINVAL_EXE}" 
                --validate 
                "${CMAKE_BINARY_DIR}/plugins/EngineField/EngineField_artefacts/$<CONFIG>/VST3/EngineField.vst3"
                --strictness-level 10
                --timeout-ms 120000
            COMMENT "Validating EngineField.vst3 with pluginval"
            DEPENDS EngineField_VST3
        )
    else()
        message(WARNING "PLUGINVAL_EXE specified but file not found: ${PLUGINVAL_EXE}")
    endif()
endif()

# Future test support
# enable_testing()
# add_subdirectory(tests)
