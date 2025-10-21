#pragma once
#include <juce_gui_basics/juce_gui_basics.h>
#include "Colors.h"

namespace engine::ui
{
    /**
     * Minimal, high-contrast look and feel for Engine plugin suite.
     * Ultra clean buttons, sliders, and toggles - no gradients, no shadows.
     */
    class EngineLookAndFeel : public juce::LookAndFeel_V4
    {
    public:
        EngineLookAndFeel()
        {
            // Set clean default colors
            setColour(juce::ResizableWindow::backgroundColourId, BG);
            setColour(juce::TextButton::buttonColourId, DIM);
            setColour(juce::TextButton::buttonOnColourId, ACCENT);
            setColour(juce::TextButton::textColourOffId, TEXT);
            setColour(juce::TextButton::textColourOnId, BG);
            setColour(juce::Slider::backgroundColourId, DIM);
            setColour(juce::Slider::thumbColourId, ACCENT);
            setColour(juce::Slider::trackColourId, ACCENT);
        }

        // Clean rectangular buttons (no rounded corners)
        void drawButtonBackground(juce::Graphics& g,
                                 juce::Button& button,
                                 const juce::Colour& backgroundColour,
                                 bool shouldDrawButtonAsHighlighted,
                                 bool shouldDrawButtonAsDown) override
        {
            auto bounds = button.getLocalBounds().toFloat();

            auto baseColour = backgroundColour;
            if (shouldDrawButtonAsDown || button.getToggleState())
                baseColour = ACCENT;
            else if (shouldDrawButtonAsHighlighted)
                baseColour = backgroundColour.brighter(0.2f);

            g.setColour(baseColour);
            g.fillRect(bounds);

            // Thin border
            g.setColour(baseColour.brighter(0.3f));
            g.drawRect(bounds, 1.0f);
        }

        // Horizontal linear slider (gradient bar)
        void drawLinearSlider(juce::Graphics& g,
                            int x, int y, int width, int height,
                            float sliderPos,
                            float /*minSliderPos*/, float /*maxSliderPos*/,
                            juce::Slider::SliderStyle style,
                            juce::Slider& slider) override
        {
            if (style != juce::Slider::LinearHorizontal)
            {
                juce::LookAndFeel_V4::drawLinearSlider(g, x, y, width, height,
                                                       sliderPos, 0, 0, style, slider);
                return;
            }

            auto bounds = juce::Rectangle<float>((float)x, (float)y, (float)width, (float)height);

            // Background track (full width, dim)
            g.setColour(DIM);
            g.fillRect(bounds);

            // Filled portion (gradient from dim to accent)
            auto filledWidth = sliderPos - x;
            if (filledWidth > 0)
            {
                juce::ColourGradient gradient(DIM, (float)x, bounds.getCentreY(),
                                             ACCENT, sliderPos, bounds.getCentreY(),
                                             false);
                g.setGradientFill(gradient);
                g.fillRect(bounds.withWidth(filledWidth));
            }

            // Thumb (thin vertical line at sliderPos)
            g.setColour(ACCENT.brighter(0.5f));
            g.fillRect(sliderPos - 1.0f, (float)y, 2.0f, (float)height);
        }

        // Clean label text
        void drawLabel(juce::Graphics& g, juce::Label& label) override
        {
            g.setColour(label.findColour(juce::Label::textColourId));
            g.setFont(getLabelFont(label));
            g.drawText(label.getText(),
                      label.getLocalBounds(),
                      label.getJustificationType(),
                      false);
        }

        juce::Font getLabelFont(juce::Label&) override
        {
            return juce::FontOptions(12.0f);
        }
    };
}
