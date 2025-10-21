#pragma once
#include <juce_gui_basics/juce_gui_basics.h>

namespace engine::ui
{
    // High contrast: Dark charcoal + white + single accent (no neon)
    inline constexpr juce::uint32 BLACK_RAW      = 0xFF0A0A0A;  // Very dark charcoal (not pure black)
    inline constexpr juce::uint32 WHITE_RAW      = 0xFFFFFFFF;  // Pure white
    inline constexpr juce::uint32 LIGHT_GRAY_RAW = 0xFFCCCCCC;  // Light gray
    inline constexpr juce::uint32 MID_GRAY_RAW   = 0xFF666666;  // Medium gray
    inline constexpr juce::uint32 DARK_GRAY_RAW  = 0xFF222222;  // Dark gray
    inline constexpr juce::uint32 ACCENT_RED_RAW = 0xFFDD0000;  // Strong red (not neon)
    inline constexpr juce::uint32 ACCENT_ORG_RAW = 0xFFFF6600;  // Strong orange

    // JUCE Colour wrappers
    inline const juce::Colour BG           {BLACK_RAW};
    inline const juce::Colour TEXT         {WHITE_RAW};
    inline const juce::Colour TEXT_DIM     {LIGHT_GRAY_RAW};
    inline const juce::Colour GRID         {MID_GRAY_RAW};
    inline const juce::Colour DIM          {DARK_GRAY_RAW};  // Alias for backwards compatibility
    inline const juce::Colour ACCENT       {ACCENT_RED_RAW};
    inline const juce::Colour ACCENT2      {ACCENT_ORG_RAW};
    inline const juce::Colour DARK_PANEL   {DARK_GRAY_RAW};

    // Semantic colors
    inline const juce::Colour POLE_ACTIVE = ACCENT;
    inline const juce::Colour POLE_TRAIL  = ACCENT.withAlpha(0.4f);
    inline const juce::Colour UNIT_CIRCLE = TEXT.withAlpha(0.4f);
    inline const juce::Colour VGA_YELLOW  = ACCENT2; // For indicators

    // Gradient helper
    inline juce::ColourGradient createPlasmaGradient(juce::Rectangle<float> bounds)
    {
        juce::ColourGradient grad(ACCENT, bounds.getTopLeft(),
                                 ACCENT2, bounds.getBottomRight(), false);
        return grad;
    }
}
