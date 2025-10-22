#include "FieldWaveformUI.h"

#include "../FieldProcessor.h"
#include "../parameters.h"

#include <BinaryData.h>

namespace engine::ui
{
    namespace
    {
        constexpr int kTimerHz = 60;
        constexpr int kBarWidthPx = 6;
        constexpr int kBarGapPx = 2;
        constexpr float kDripBase = 0.2f;
        constexpr float kDripRange = 0.8f;

        inline float clamp01(float value) noexcept
        {
            return juce::jlimit(0.0f, 1.0f, value);
        }
    }

    FieldWaveformEditor::FieldWaveformEditor(FieldProcessor& processor,
                                             juce::AudioProcessorValueTreeState& state)
        : juce::AudioProcessorEditor(&processor), processorRef_(processor)
    {
        setOpaque(true);
        setSize(420, 560);

        skinOff_ = juce::Drawable::createFromImageData(BinaryData::_1_svg, BinaryData::_1_svgSize);
        skinOn_  = juce::Drawable::createFromImageData(BinaryData::_2_svg, BinaryData::_2_svgSize);

        mixParam_       = state.getRawParameterValue(enginefield::params::mixId);
        characterParam_ = state.getRawParameterValue(enginefield::params::characterId);
        effectParam_    = state.getRawParameterValue(enginefield::params::effectModeId);

        // Configure MIX slider (horizontal, top-left)
        mixSlider_.setName("MIX");
        mixSlider_.setSliderStyle(juce::Slider::LinearHorizontal);
        mixSlider_.setTextBoxStyle(juce::Slider::NoTextBox, false, 0, 0);
        mixSlider_.setLookAndFeel(&fieldLNF_);
        addAndMakeVisible(mixSlider_);

        // Configure CHARACTER slider (horizontal, bottom)
        characterSlider_.setName("CHARACTER");
        characterSlider_.setSliderStyle(juce::Slider::LinearHorizontal);
        characterSlider_.setTextBoxStyle(juce::Slider::NoTextBox, false, 0, 0);
        characterSlider_.setLookAndFeel(&fieldLNF_);
        addAndMakeVisible(characterSlider_);

        // Configure EFFECT button (toggle, top-right)
        effectButton_.setName("EFFECT");
        effectButton_.setButtonText("");  // SVG shows label
        effectButton_.setClickingTogglesState(true);
        effectButton_.setLookAndFeel(&fieldLNF_);
        addAndMakeVisible(effectButton_);

        // Create APVTS attachments (two-way parameter binding)
        mixAttachment_ = std::make_unique<juce::AudioProcessorValueTreeState::SliderAttachment>(
            state, enginefield::params::mixId, mixSlider_);
        characterAttachment_ = std::make_unique<juce::AudioProcessorValueTreeState::SliderAttachment>(
            state, enginefield::params::characterId, characterSlider_);
        effectAttachment_ = std::make_unique<juce::AudioProcessorValueTreeState::ButtonAttachment>(
            state, enginefield::params::effectModeId, effectButton_);

        startTimerHz(kTimerHz);
    }

    FieldWaveformEditor::~FieldWaveformEditor() = default;

    void FieldWaveformEditor::timerCallback()
    {
        processorRef_.getWaveformPeaks(waveformPeaks_);
        currentLevel_ = clamp01(processorRef_.getCurrentLevel());
        repaint();
    }

    void FieldWaveformEditor::paint(juce::Graphics& g)
    {
        g.fillAll(RetroPalette::kBackground);

        const float mixAlpha   = mixParam_       ? clamp01(mixParam_->load() * 0.01f)       : 1.0f;
        const float character  = characterParam_ ? clamp01(characterParam_->load() * 0.01f) : 0.0f;
        const bool effectOn    = effectParam_    && effectParam_->load() > 0.5f;

        if (auto* skin = effectOn ? skinOn_.get() : skinOff_.get())
            skin->draw(g, 1.0f);

        auto viewport = viewportPx_;
        juce::Graphics::ScopedSaveState clipper(g);
        g.reduceClipRegion(viewport);

        if (!effectOn)
        {
            const int y = viewport.getCentreY() - 1;
            g.setColour(RetroPalette::kBaseline.withAlpha(mixAlpha));
            for (int x = viewport.getX(); x < viewport.getRight(); x += 8)
            {
                const int seg = juce::jmin(x + 4, viewport.getRight());
                g.fillRect(x, y, seg - x, 2);
            }
        }

        drawWaveform(g, viewport, mixAlpha, character);
        drawPeakTracer(g, viewport, mixAlpha);
        drawAlivePulse(g, viewport);
    }

    void FieldWaveformEditor::drawWaveform(juce::Graphics& g, juce::Rectangle<int> viewport, float alpha, float character) const
    {
        const int count = static_cast<int>(waveformPeaks_.size());
        if (count == 0 || alpha <= 0.0f)
            return;

        const float spacing = static_cast<float>(viewport.getWidth()) / static_cast<float>(count);
        const int barWidth = juce::jmax(1, juce::roundToInt(spacing * 0.7f));
        const float dripFactor = kDripBase + character * kDripRange;
        const int baselineY = viewport.getCentreY();
        const float halfH = viewport.getHeight() * 0.44f;

        g.setColour(RetroPalette::kBarFill.withAlpha(alpha));

        float x = static_cast<float>(viewport.getX());
        for (int i = 0; i < count; ++i, x += spacing)
        {
            const float peak = clamp01(waveformPeaks_[i]);
            const int xi = juce::roundToInt(x);
            const int above = juce::roundToInt(peak * halfH);
            const int drip  = juce::roundToInt(peak * halfH * dripFactor);

            if (above > 0)
                g.fillRect(xi, baselineY - above, barWidth, juce::jmax(1, above));
            if (drip > 0)
                g.fillRect(xi, baselineY, barWidth, juce::jmax(1, drip));
        }
    }

    void FieldWaveformEditor::drawPeakTracer(juce::Graphics& g, juce::Rectangle<int> viewport, float alpha) const
    {
        const int count = static_cast<int>(waveformPeaks_.size());
        if (count == 0 || alpha <= 0.0f)
            return;

        const float spacing = static_cast<float>(viewport.getWidth()) / static_cast<float>(count);
        const int baselineY = viewport.getCentreY();
        const float halfH = viewport.getHeight() * 0.44f;

        juce::Path path;
        bool started = false;
        float x = static_cast<float>(viewport.getX());
        for (int i = 0; i < count; ++i, x += spacing)
        {
            const float peak = clamp01(waveformPeaks_[i]);
            const float xi = x + spacing * 0.5f;
            const float yi = static_cast<float>(baselineY) - peak * halfH;
            if (!started)
            {
                path.startNewSubPath(xi, yi);
                started = true;
            }
            else
            {
                path.lineTo(xi, yi);
            }
        }
        if (!started)
            return;

        g.setColour(RetroPalette::kPeakTracer.withAlpha(alpha * 0.45f));
        g.strokePath(path, juce::PathStrokeType(4.0f));
        g.setColour(RetroPalette::kPeakTracer.withAlpha(alpha));
        g.strokePath(path, juce::PathStrokeType(2.0f));
    }

    void FieldWaveformEditor::drawAlivePulse(juce::Graphics& g, juce::Rectangle<int> viewport) const
    {
        juce::Rectangle<int> pulse(viewport.getRight() - 11, viewport.getY() + 8, 3, 10);
        g.setColour(RetroPalette::kBaseline);
        g.fillRect(pulse);
    }

    void FieldWaveformEditor::resized()
    {
        // Position interactive controls to overlay SVG graphics (pixel-perfect alignment)
        // Coordinates based on PNG design reference and SVG element positions
        // Window size: 420×560px (set in constructor line 29)

        // MIX slider - top-left horizontal bar
        mixSlider_.setBounds(40, 50, 110, 24);

        // EFFECT button - top-right toggle (v1.0.1: fixed out-of-bounds bug)
        effectButton_.setBounds(285, 40, 100, 35);  // Was 390, causing right edge @ 495px > 420px window

        // CHARACTER slider - bottom horizontal bar
        characterSlider_.setBounds(40, 500, 340, 24);  // Aligned x with MIX, width scaled to fit
    }
} // namespace engine::ui
