# Plugin configuration
set(PRODUCT_NAME "EngineField")
set(TARGET_NAME ${PRODUCT_NAME})
set(COMPANY_NAME "Engine Audio")
set(BUNDLE_ID "com.engineaudio.EngineField")

# Determine plugin formats to build
set(FORMATS "")
if(BUILD_VST3)
    list(APPEND FORMATS VST3)
endif()
if(BUILD_STANDALONE)
    list(APPEND FORMATS Standalone)
endif()

if(NOT FORMATS)
    message(FATAL_ERROR "No plugin formats selected to build. Enable at least one format (VST3 or Standalone)")
endif()

# Configure JUCE plugin
option(FIELD_INSTALL_AFTER_BUILD "Copy VST3 to the system VST3 folder after build" OFF)

juce_add_plugin(${TARGET_NAME}
    VERSION ${PROJECT_VERSION}
    COMPANY_NAME ${COMPANY_NAME}
    COMPANY_WEBSITE "https://engineaudio.com"
    COMPANY_EMAIL "support@engineaudio.com"
    BUNDLE_ID ${BUNDLE_ID}
    
    # Plugin characteristics
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    IS_MIDI_EFFECT FALSE
    
    # Build settings
    COPY_PLUGIN_AFTER_BUILD ${FIELD_INSTALL_AFTER_BUILD}
    PLUGIN_MANUFACTURER_CODE EngA
    PLUGIN_CODE EnFd
    FORMATS ${FORMATS}
    VST3_CATEGORIES "Fx"
    PRODUCT_NAME ${PRODUCT_NAME}
)

# Source files
target_sources(${TARGET_NAME}
    PRIVATE
        Source/PluginMain.cpp
        Source/FieldProcessor.cpp
        Source/FieldProcessor.h
        Source/parameters.h
        Source/dsp/ZPlaneFilter.h
        Source/dsp/EMUAuthenticTables.h
        Source/dsp/EnvelopeFollower.h
        Source/ui/FieldWaveformUI.cpp
        Source/ui/FieldWaveformUI.h
)

# Plugin-specific compile definitions
target_compile_definitions(${TARGET_NAME}
    PRIVATE
        JUCE_WEB_BROWSER=0
        JUCE_USE_CURL=0
        JUCE_VST3_CAN_REPLACE_VST2=0
        JUCE_STRICT_REFCOUNTEDPOINTER=1
        EMU_DSP_LOCKED=1
        # Versioning
        PLUGIN_VERSION="${PROJECT_VERSION}"
        # Enable console for standalone debug
        $<$<BOOL:${BUILD_STANDALONE}>:JUCE_APPLICATION_NAME_STRING="${PRODUCT_NAME}">
)

# JUCE dependencies
target_link_libraries(${TARGET_NAME}
    PRIVATE
        juce::juce_core
        juce::juce_data_structures
        juce::juce_events
        juce::juce_graphics
        juce::juce_gui_basics
        juce::juce_gui_extra
        juce::juce_audio_basics
        juce::juce_audio_devices
        juce::juce_audio_formats
        juce::juce_audio_processors
        juce::juce_audio_utils
        juce::juce_dsp
)

# Warning configuration
if(MSVC)
    target_compile_options(${TARGET_NAME} 
        PRIVATE 
            /WX     # Warnings as errors
            /wd4458 # Suppress declaration hides class member
    )
else()
    target_compile_options(${TARGET_NAME} 
        PRIVATE 
            -Werror # Warnings as errors
            -Wno-deprecated-declarations
    )
endif()

# IDE source groups for better organization
source_group(TREE "${CMAKE_CURRENT_SOURCE_DIR}/Source" PREFIX "Source Files" FILES ${SOURCES})

# Embed skin assets (1.svg: EFFECT off, 2.svg: EFFECT on)
juce_add_binary_data(EngineFieldAssets
    SOURCES
        ${CMAKE_SOURCE_DIR}/1.svg
        ${CMAKE_SOURCE_DIR}/2.svg
)

target_link_libraries(${TARGET_NAME} PRIVATE EngineFieldAssets)
