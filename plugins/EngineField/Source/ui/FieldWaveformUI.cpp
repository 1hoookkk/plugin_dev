#include "FieldWaveformUI.h"

#include "../FieldProcessor.h"
#include "../parameters.h"

namespace engine::ui
{
    namespace
    {
        constexpr int kTimerHz = 30;
        constexpr int kBarWidth = 6;
        constexpr int kBarGap = 2;
        constexpr int kViewportPadding = 24;
        constexpr int kViewportFrameThickness = 8;

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
        setLookAndFeel(&lookAndFeel_);
        setSize(704, 980);

        mixLabel_.setText("MIX", juce::dontSendNotification);
        mixLabel_.setJustificationType(juce::Justification::centredLeft);
        mixLabel_.setColour(juce::Label::textColourId, RetroPalette::kViewportFrame);
        mixLabel_.setFont(juce::FontOptions(26.0f, juce::Font::bold));
        addAndMakeVisible(mixLabel_);

        mixSlider_.setName("MIX");
        mixSlider_.setSliderStyle(juce::Slider::LinearHorizontal);
        mixSlider_.setTextBoxStyle(juce::Slider::NoTextBox, false, 0, 0);
        mixSlider_.setRange(0.0, 100.0, 0.01);
        addAndMakeVisible(mixSlider_);

        characterLabel_.setText("CHARACTER", juce::dontSendNotification);
        characterLabel_.setJustificationType(juce::Justification::centredLeft);
        characterLabel_.setColour(juce::Label::textColourId, RetroPalette::kViewportFrame);
        characterLabel_.setFont(juce::FontOptions(28.0f, juce::Font::bold));
        addAndMakeVisible(characterLabel_);

        characterSlider_.setName("CHARACTER");
        characterSlider_.setSliderStyle(juce::Slider::LinearHorizontal);
        characterSlider_.setTextBoxStyle(juce::Slider::NoTextBox, false, 0, 0);
        characterSlider_.setRange(0.0, 100.0, 0.01);
        addAndMakeVisible(characterSlider_);

        effectButton_.setName("EFFECT");
        effectButton_.setButtonText("EFFECT");
        effectButton_.setClickingTogglesState(true);
        addAndMakeVisible(effectButton_);

        mixAttachment_ = std::make_unique<SliderAttachment>(state, enginefield::params::mixId, mixSlider_);
        characterAttachment_ = std::make_unique<SliderAttachment>(state, enginefield::params::characterId, characterSlider_);
        effectAttachment_ = std::make_unique<ButtonAttachment>(state, enginefield::params::effectModeId, effectButton_);

        startTimerHz(kTimerHz);
        resized(); // initialise bounds
    }

    FieldWaveformEditor::~FieldWaveformEditor()
    {
        setLookAndFeel(nullptr);
    }

    void FieldWaveformEditor::timerCallback()
    {
        processorRef_.getWaveformPeaks(waveformPeaks_);
        currentLevel_ = clamp01(processorRef_.getCurrentLevel());
        repaint(viewportBounds_.expanded(8));
    }

    void FieldWaveformEditor::paint(juce::Graphics& g)
    {
        g.fillAll(RetroPalette::kBackground);

        g.setColour(RetroPalette::kViewportFrame);
        g.drawRect(getLocalBounds(), 4);

        if (!viewportBounds_.isEmpty())
            drawViewport(g, viewportBounds_);
    }

    void FieldWaveformEditor::drawViewport(juce::Graphics& g, juce::Rectangle<int> bounds)
    {
        auto frame = bounds;
        g.setColour(RetroPalette::kViewportFrame);
        g.drawRect(frame, kViewportFrameThickness);

        auto area = frame.reduced(kViewportFrameThickness + kViewportPadding);
        area = area.withTrimmedBottom(16);

        g.setColour(RetroPalette::kViewportBackground);
        g.fillRect(area);

        const auto areaFloat = area.toFloat();

        drawWaveform(g, areaFloat);
        drawPeakTracer(g, areaFloat);
        drawLevelMarker(g, areaFloat);
    }

    void FieldWaveformEditor::drawWaveform(juce::Graphics& g, juce::Rectangle<float> area) const
    {
        const int totalBars = static_cast<int>(waveformPeaks_.size());
        if (totalBars == 0)
            return;

        const float spacing = static_cast<float>(kBarWidth + kBarGap);
        const float totalWidth = totalBars * spacing - static_cast<float>(kBarGap);
        const float startXFloat = area.getX() + juce::jmax(0.0f, (area.getWidth() - totalWidth) * 0.5f);
        const int startX = juce::roundToInt(startXFloat);
        const int baselineY = juce::roundToInt(area.getCentreY());
        const float halfHeightF = area.getHeight() * 0.44f;
        const float dripFactor = 0.2f + clamp01(static_cast<float>(characterSlider_.getValue() * 0.01)) * 0.8f;

        g.setColour(RetroPalette::kBarFill);

        for (size_t i = 0; i < waveformPeaks_.size(); ++i)
        {
            const float peak = clamp01(waveformPeaks_[i]);
            const float aboveF = peak * halfHeightF;
            const float dripF = aboveF * dripFactor;
            const int x = startX + juce::roundToInt(static_cast<float>(i) * spacing);
            const int above = juce::roundToInt(aboveF);
            const int drip = juce::roundToInt(dripF);

            if (above > 0)
                g.fillRect(x, baselineY - above, kBarWidth, juce::jmax(1, above));

            if (drip > 0)
                g.fillRect(x, baselineY, kBarWidth, juce::jmax(1, drip));
        }
    }

    void FieldWaveformEditor::drawPeakTracer(juce::Graphics& g, juce::Rectangle<float> area) const
    {
        const int totalBars = static_cast<int>(waveformPeaks_.size());
        if (totalBars == 0)
            return;

        juce::Path tracer;
        const float spacing = static_cast<float>(kBarWidth + kBarGap);
        const float totalWidth = totalBars * spacing - static_cast<float>(kBarGap);
        const float startX = static_cast<float>(juce::roundToInt(area.getX() + juce::jmax(0.0f, (area.getWidth() - totalWidth) * 0.5f)))
                            + static_cast<float>(kBarWidth) * 0.5f;
        const float baselineY = static_cast<float>(juce::roundToInt(area.getCentreY()));
        const float halfHeightF = area.getHeight() * 0.44f;

        bool started = false;
        for (size_t i = 0; i < waveformPeaks_.size(); ++i)
        {
            const float peak = clamp01(waveformPeaks_[i]);
            const float x = startX + static_cast<float>(juce::roundToInt(static_cast<float>(i) * spacing));
            const float y = baselineY - peak * halfHeightF;

            if (!started)
            {
                tracer.startNewSubPath(x, y);
                started = true;
            }
            else
            {
                tracer.lineTo(x, y);
            }
        }

        if (!started)
            return;

        g.setColour(RetroPalette::kPeakTracer.withAlpha(0.45f));
        g.strokePath(tracer, juce::PathStrokeType(4.0f));

        g.setColour(RetroPalette::kPeakTracer);
        g.strokePath(tracer, juce::PathStrokeType(2.0f));
    }

    void FieldWaveformEditor::drawLevelMarker(juce::Graphics& g, juce::Rectangle<float> area) const
    {
        const int markerWidth = 36;
        const int markerHeight = 5;

        const float minYF = area.getY() + 16.0f;
        const float maxYF = area.getCentreY() - 24.0f;
        const int y = juce::roundToInt(juce::jmap(1.0f - currentLevel_, 0.0f, 1.0f, minYF, maxYF));
        const int x = juce::roundToInt(area.getX() + (area.getWidth() - static_cast<float>(markerWidth)) * 0.5f);

        g.setColour(RetroPalette::kBaseline);
        g.fillRect(x, y, markerWidth, markerHeight);
    }    void FieldWaveformEditor::resized()
    {
        auto bounds = getLocalBounds().reduced(24);

        auto topRow = bounds.removeFromTop(120);
        auto effectArea = topRow.removeFromRight(180).reduced(12);
        effectButton_.setBounds(effectArea);

        mixLabel_.setBounds(topRow.removeFromTop(36));
        mixSlider_.setBounds(topRow.removeFromTop(32));

        auto bottomRow = bounds.removeFromBottom(140);
        characterLabel_.setBounds(bottomRow.removeFromTop(40));
        characterSlider_.setBounds(bottomRow.removeFromTop(44));

        viewportBounds_ = bounds;
    }
} // namespace engine::ui












