#pragma once
#include <juce_gui_basics/juce_gui_basics.h>
#include "Colors.h"

namespace engine::ui
{
    /**
     * Horizontal gradient slider for mix/blend controls.
     * Visual: [██████░░░░] with smooth gradient from dim to accent.
     */
    class HorizontalMixSlider : public juce::Slider
    {
    public:
        HorizontalMixSlider()
        {
            setSliderStyle(juce::Slider::LinearHorizontal);
            setTextBoxStyle(juce::Slider::NoTextBox, false, 0, 0);
            setRange(0.0, 100.0, 0.1);
            setValue(100.0);
        }

        void paint(juce::Graphics& g) override
        {
            auto bounds = getLocalBounds().toFloat();

            // Background track (dim)
            g.setColour(DIM);
            g.fillRect(bounds);

            // Filled portion (gradient)
            float proportion = (float)((getValue() - getMinimum()) / (getMaximum() - getMinimum()));
            float filledWidth = bounds.getWidth() * proportion;

            if (filledWidth > 0.0f)
            {
                juce::ColourGradient gradient(DIM, bounds.getX(), bounds.getCentreY(),
                                             ACCENT, bounds.getX() + filledWidth, bounds.getCentreY(),
                                             false);
                g.setGradientFill(gradient);
                g.fillRect(bounds.withWidth(filledWidth));
            }

            // Thumb (thin vertical line)
            float thumbX = bounds.getX() + filledWidth;
            g.setColour(ACCENT.brighter(0.5f));
            g.fillRect(thumbX - 1.0f, bounds.getY(), 2.0f, bounds.getHeight());

            // Label overlay (right-aligned percentage)
            g.setColour(TEXT);
            g.setFont(juce::FontOptions(11.0f));
            juce::String label = juce::String((int)getValue()) + "%";
            g.drawText(label, bounds.reduced(4.0f), juce::Justification::centredRight, false);
        }
    };

    /**
     * Minimal toggle button for bypass.
     * Visual: [ ] or [X] with simple frame.
     */
    class MinimalToggle : public juce::ToggleButton
    {
    public:
        explicit MinimalToggle(const juce::String& text = {})
        {
            setButtonText(text);
        }

        void paintButton(juce::Graphics& g, bool shouldDrawButtonAsHighlighted, bool shouldDrawButtonAsDown) override
        {
            auto bounds = getLocalBounds().toFloat();

            // Frame
            auto baseColour = getToggleState() ? ACCENT : DIM;
            if (shouldDrawButtonAsHighlighted || shouldDrawButtonAsDown)
                baseColour = baseColour.brighter(0.2f);

            g.setColour(baseColour);
            g.drawRect(bounds, 1.0f);

            // Fill if toggled
            if (getToggleState())
            {
                g.setColour(ACCENT.withAlpha(0.2f));
                g.fillRect(bounds.reduced(2.0f));
            }

            // Text label
            if (getButtonText().isNotEmpty())
            {
                g.setColour(TEXT);
                g.setFont(juce::FontOptions(11.0f));
                g.drawText(getButtonText(), bounds, juce::Justification::centred, false);
            }
        }
    };

    /**
     * Simple text label with engine styling.
     */
    class MinimalLabel : public juce::Label
    {
    public:
        explicit MinimalLabel(const juce::String& text = {})
        {
            setText(text, juce::dontSendNotification);
            setFont(juce::FontOptions(11.0f));
            setColour(juce::Label::textColourId, TEXT);
            setJustificationType(juce::Justification::centredLeft);
        }
    };
}
