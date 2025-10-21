#pragma once
#include <juce_gui_basics/juce_gui_basics.h>

namespace engine::ui
{
    /**
     * Custom LookAndFeel for Field plugin matching SVG design specifications.
     * Renders sliders and buttons with custom graphics.
     */
    class FieldLookAndFeel : public juce::LookAndFeel_V4
    {
    public:
        FieldLookAndFeel()
        {
            setColour(juce::Slider::trackColourId, juce::Colour(0xFF1F3750)); // Dark track
            setColour(juce::Slider::thumbColourId, juce::Colour(0xFFF9F034)); // Yellow indicator
        }

        void drawLinearSlider(juce::Graphics& g, int x, int y, int width, int height,
                             float sliderPos, float minSliderPos, float maxSliderPos,
                             juce::Slider::SliderStyle style, juce::Slider& slider) override
        {
            if (slider.getName() == "CHARACTER")
            {
                drawCharacterSlider(g, x, y, width, height, sliderPos);
            }
            else if (slider.getName() == "MIX")
            {
                drawMixSlider(g, x, y, width, height, sliderPos);
            }
            else
            {
                // Fallback to default rendering
                juce::LookAndFeel_V4::drawLinearSlider(g, x, y, width, height,
                                                      sliderPos, minSliderPos, maxSliderPos,
                                                      style, slider);
            }
        }

        void drawButtonBackground(juce::Graphics& g, juce::Button& button,
                                 const juce::Colour& backgroundColour,
                                 bool shouldDrawButtonAsHighlighted,
                                 bool shouldDrawButtonAsDown) override
        {
            if (button.getName() == "EFFECT")
            {
                drawEffectButton(g, button);
                return;
            }

            // Fallback to default
            juce::LookAndFeel_V4::drawButtonBackground(g, button, backgroundColour,
                                                      shouldDrawButtonAsHighlighted,
                                                      shouldDrawButtonAsDown);
        }

        void drawButtonText(juce::Graphics& g, juce::TextButton& button,
                           bool shouldDrawButtonAsHighlighted, bool shouldDrawButtonAsDown) override
        {
            if (button.getName() == "EFFECT")
            {
                g.setFont(juce::FontOptions(12.0f, juce::Font::bold));

                // Yellow text when on, white when off
                if (button.getToggleState())
                    g.setColour(juce::Colour(0xFFF9F034)); // Yellow
                else
                    g.setColour(juce::Colour(0xFFF0F5FB)); // Light blue/white

                g.drawText("EFFECT", button.getLocalBounds(),
                          juce::Justification::centred, true);
                return;
            }

            // Fallback to default
            juce::LookAndFeel_V4::drawButtonText(g, button,
                                                shouldDrawButtonAsHighlighted,
                                                shouldDrawButtonAsDown);
        }

    private:
        void drawCharacterSlider(juce::Graphics& g, int x, int y, int width, int height, float sliderPos)
        {
            // Pixel-snapped rectangle (integer coordinates for crisp edges)
            auto bounds = juce::Rectangle<int>(x, y, width, height);

            // Draw dark blue background with light blue stroke
            g.setColour(juce::Colour(0xFF2C4C70)); // Dark blue background
            g.fillRect(bounds);

            g.setColour(juce::Colour(0xFFF0F5FB)); // Light blue stroke
            g.drawRect(bounds, 2);

            // Draw yellow position indicator
            int indicatorWidth = static_cast<int>(sliderPos) - x;
            if (indicatorWidth > 8) // Only draw if wide enough
            {
                auto indicatorBounds = bounds.withWidth(indicatorWidth).reduced(4);

                g.setColour(juce::Colour(0xFFF9F034)); // Yellow fill
                g.fillRect(indicatorBounds);

                g.setColour(juce::Colour(0xFF111111)); // Black stroke
                g.drawRect(indicatorBounds, 3);
            }
        }

        void drawMixSlider(juce::Graphics& g, int x, int y, int width, int height, float sliderPos)
        {
            // Pixel-snapped rectangle (integer coordinates for crisp edges)
            auto bounds = juce::Rectangle<int>(x, y, width, height);

            // Draw dark track background
            g.setColour(juce::Colour(0xFF1F3750)); // Dark background
            g.fillRect(bounds);

            g.setColour(juce::Colour(0xFF111111)); // Black stroke
            g.drawRect(bounds, 2);

            // Draw yellow fill indicator
            int indicatorWidth = static_cast<int>(sliderPos) - x;
            if (indicatorWidth > 4)
            {
                auto indicatorBounds = bounds.withWidth(indicatorWidth).reduced(2);

                g.setColour(juce::Colour(0xFFF9F034)); // Yellow fill
                g.fillRect(indicatorBounds);
            }
        }

        void drawEffectButton(juce::Graphics& g, juce::Button& button)
        {
            // Pixel-snapped rectangle (integer coordinates for crisp edges)
            auto bounds = button.getLocalBounds();

            if (button.getToggleState())
            {
                // ON state: Red fill with black stroke
                g.setColour(juce::Colour(0xFFB02020)); // Red fill
                g.fillRect(bounds.reduced(2));

                g.setColour(juce::Colour(0xFF000000)); // Black stroke
                g.drawRect(bounds.reduced(2), 4);
            }
            else
            {
                // OFF state: Blue fill with black stroke
                g.setColour(juce::Colour(0xFF2D6DA9)); // Blue fill (matches background)
                g.fillRect(bounds.reduced(2));

                g.setColour(juce::Colour(0xFF000000)); // Black stroke (4px)
                g.drawRect(bounds.reduced(2), 4);

                // Light blue outer stroke (2px)
                g.setColour(juce::Colour(0xFFF0F5FB)); // Light blue stroke
                g.drawRect(bounds.reduced(1), 2);
            }
        }

        JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(FieldLookAndFeel)
    };
}
