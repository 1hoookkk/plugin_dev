
#pragma once
#include <juce_audio_processors/juce_audio_processors.h>

namespace enginefield::params
{
    static constexpr auto characterId = "character";
    static constexpr auto mixId       = "mix";
    static constexpr auto gainId      = "gain";
    static constexpr auto bypassId    = "bypass";
    static constexpr auto testToneId  = "testTone";
    static constexpr auto effectModeId = "effectMode";

    inline juce::AudioProcessorValueTreeState::ParameterLayout createLayout()
    {
        std::vector<std::unique_ptr<juce::RangedAudioParameter>> ps;
        using juce::ParameterID;

        ps.push_back(std::make_unique<juce::AudioParameterFloat>(
            ParameterID{ characterId, 1 }, "Character",
            juce::NormalisableRange<float>{ 0.0f, 100.0f, 0.01f }, 50.0f,
            juce::AudioParameterFloatAttributes().withLabel("%")
        ));

        ps.push_back(std::make_unique<juce::AudioParameterFloat>(
            ParameterID{ mixId, 1 }, "Mix",
            juce::NormalisableRange<float>{ 0.0f, 100.0f, 0.01f }, 100.0f,
            juce::AudioParameterFloatAttributes().withLabel("%")
        ));

        ps.push_back(std::make_unique<juce::AudioParameterFloat>(
            ParameterID{ gainId, 1 }, "Output",
            juce::NormalisableRange<float>{ -12.0f, 12.0f, 0.01f }, 0.0f,
            juce::AudioParameterFloatAttributes().withLabel("dB")
        ));

        ps.push_back(std::make_unique<juce::AudioParameterBool>(
            ParameterID{ bypassId, 1 }, "Bypass", false
        ));

        ps.push_back(std::make_unique<juce::AudioParameterBool>(
            ParameterID{ testToneId, 1 }, "Test Tone (440Hz)", false
        ));

        ps.push_back(std::make_unique<juce::AudioParameterBool>(
            ParameterID{ effectModeId, 1 }, "EFFECT (Wet Solo)", false
        ));

        return { ps.begin(), ps.end() };
    }
}
