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
        inline const juce::Colour kBackground { 0xFF2D6DA9 };
        inline const juce::Colour kViewportBackground { 0xFF000000 };
        inline const juce::Colour kBaseline { 0xFF59B850 };
        inline const juce::Colour kBarFill { 0xFFE8D348 };
        inline const juce::Colour kPeakTracer { 0xFFC3FF00 };
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
        void timerCallback() override;
        void drawWaveform(juce::Graphics& g, juce::Rectangle<int> viewport, float alpha, float character) const;
        void drawPeakTracer(juce::Graphics& g, juce::Rectangle<int> viewport, float alpha) const;
        void drawAlivePulse(juce::Graphics& g, juce::Rectangle<int> viewport) const;

        FieldProcessor& processorRef_;

        std::atomic<float>* mixParam_ { nullptr };
        std::atomic<float>* characterParam_ { nullptr };
        std::atomic<float>* effectParam_ { nullptr };

        std::unique_ptr<juce::Drawable> skinOff_;
        std::unique_ptr<juce::Drawable> skinOn_;

        const juce::Rectangle<int> viewportPx_ { 16, 80, 388, 348 };

        std::array<float, engine::viz::kWaveformBarCount> waveformPeaks_ {};
        float currentLevel_ = 0.0f;

        JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(FieldWaveformEditor)
    };
} // namespace engine::ui
