#pragma once

#include <juce_audio_processors/juce_audio_processors.h>
#include <juce_gui_basics/juce_gui_basics.h>
#include <array>
#include <memory>

#include "VisualisationConfig.h"

class FieldProcessor;

namespace engine::ui
{
    namespace RetroPalette
    {
        inline const juce::Colour kBackground         { 0xFF2D6DA9 };
        inline const juce::Colour kViewportBackground { 0xFF050505 };
        inline const juce::Colour kViewportFrame      { 0xFFCED9E8 };
        inline const juce::Colour kBaseline           { 0xFF59B850 };
        inline const juce::Colour kBarFill            { 0xFFE8D348 };
        inline const juce::Colour kPeakTracer         { 0xFFC3FF00 };
        inline const juce::Colour kMeterDim           { 0xFF153454 };
    }

    class FieldWaveformEditor final : public juce::AudioProcessorEditor,
                                      private juce::Timer
    {
    public:
        FieldWaveformEditor(FieldProcessor& processor,
                            juce::AudioProcessorValueTreeState& state);
        ~FieldWaveformEditor() override;

        void paint(juce::Graphics& g) override;
        void resized() override;

    private:
        using SliderAttachment = juce::AudioProcessorValueTreeState::SliderAttachment;
        using ButtonAttachment = juce::AudioProcessorValueTreeState::ButtonAttachment;

        void timerCallback() override;
        void drawViewport(juce::Graphics& g, juce::Rectangle<int> bounds);
        void drawWaveform(juce::Graphics& g, juce::Rectangle<float> area) const;
        void drawPeakTracer(juce::Graphics& g, juce::Rectangle<float> area) const;
        void drawLevelMarker(juce::Graphics& g, juce::Rectangle<float> area) const;

        FieldProcessor& processorRef_;
                FieldLookAndFeel lookAndFeel_;

        juce::Slider mixSlider_;
        juce::Slider characterSlider_;
        juce::TextButton effectButton_;
        juce::Label mixLabel_;
        juce::Label characterLabel_;

        std::unique_ptr<SliderAttachment> mixAttachment_;
        std::unique_ptr<SliderAttachment> characterAttachment_;
        std::unique_ptr<ButtonAttachment> effectAttachment_;

                std::array<float, engine::viz::kWaveformBarCount> waveformPeaks_{};
        float currentLevel_ = 0.0f;

        juce::Rectangle<int> viewportBounds_;

        JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(FieldWaveformEditor)
    };
} // namespace engine::ui



