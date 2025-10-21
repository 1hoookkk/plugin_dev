#pragma once
#include <juce_gui_basics/juce_gui_basics.h>

namespace engine::ui::glyphs
{
    // Font creation helper (simplified for Phase 1)
    inline juce::Font createAlienFont(float height)
    {
        // Use a clean sans-serif font with slight spacing adjustment
        auto font = juce::Font(juce::FontOptions().withHeight(height));
        font.setExtraKerningFactor(0.05f);  // Slightly wider spacing
        return font;
    }
}
