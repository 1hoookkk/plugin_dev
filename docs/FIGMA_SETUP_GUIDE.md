# FIELD - Figma Setup Guide

**Goal:** Design the FIELD UI (dual-trace visualizer concept) in Figma, then hand off to JUCE implementation.

---

## Step 1: Create New Figma File

1. Go to Figma → New Design File
2. Name it: **"Engine Suite - FIELD"**
3. Create pages:
   - **Design Tokens** (color/type reference)
   - **FIELD UI** (main design work)
   - **Components** (reusable elements)
   - **Developer Handoff** (annotated specs)

---

## Step 2: Import Design Tokens as Variables

### Color Variables

In Figma, create **Local Variables** (not just color styles):

**Chassis:**
- `chassis/primary` = `#3B7FC4` (Vibrant Blue)

**Viewport:**
- `viewport/base` = `#000000` (Black)

**Accent:**
- `accent/voltage-lime` = `#C3FF00` (Processed trace)
- `accent/slider-thumb` = `#E8D348` (Warm yellow)
- `accent/crt-green` = `#3DD665` (Input trace)

**Text:**
- `text/primary` = `#FFFFFF` (White)
- `text/secondary` = `#EAEAEA` (Soft white)
- `text/inverse` = `#000000` (Black on yellow)

**Border:**
- `border/hard` = `#111111` (Black separator)
- `border/soft` = `#EAEAEA` (Light separator)
- `border/thumb-outline` = `#606060` (Grey outline)

**State:**
- `state/flash` = `#3DD665` (Interaction flash)

### Typography Styles

Create **Text Styles**:

1. **Label/Primary**
   - Font: DM Sans (or Satoshi if available, fallback: Inter)
   - Size: 12px
   - Weight: Bold (700)
   - Transform: ALL CAPS
   - Color: `text/primary`

2. **Numeric/Readout**
   - Font: IBM Plex Mono (fallback: Courier New)
   - Size: 12px
   - Weight: Regular (400)
   - Color: `text/primary`

3. **Caption/Small**
   - Font: DM Sans
   - Size: 11px
   - Weight: Regular (400)
   - Color: `text/secondary`

### Spacing Variables

Create **Number Variables**:
- `spacing/unit` = 4
- `spacing/control-padding` = 16
- `spacing/slider-height` = 20
- `spacing/slider-height-hero` = 22

---

## Step 3: Create the Main Frame

### Frame Setup

1. Create a new frame: **"FIELD - Default State"**
2. Dimensions: **360×640 px** (9:16 portrait ratio)
3. Background: `chassis/primary` (#3B7FC4)

### Layout Grid (optional but helpful)

Add a 4px baseline grid:
- Type: Grid
- Size: 4px
- Color: Red at 10% opacity
- This ensures all spacing uses 4px multiples

---

## Step 4: Build the Layout Structure

### Region Breakdown

```
┌─────────────────────────────────┐  ← 0px
│  MIX        [value]    EFFECT   │
│  ═══════════════════════════════│  ← 72px (header)
├─────────────────────────────────┤
│ ┌───────────────────────────┐   │
│ │                           │   │
│ │   BLACK VIEWPORT          │   │
│ │   (waveform traces)       │   │
│ │                           │   │
│ │                           │   │
│ └───────────────────────────┘   │
├─────────────────────────────────┤  ← 584px (footer start)
│  CHARACTER  ═════════════════   │
│                           [val] │
└─────────────────────────────────┘  ← 640px
```

**Measurements:**
- Header: 0-72px (72px tall)
- Viewport: 72-584px (512px tall, with 16px padding on sides)
- Footer: 584-640px (56px tall)

---

## Step 5: Build Components (Top to Bottom)

### Header Region (0-72px)

#### MIX Label
- Text: "MIX"
- Style: `Label/Primary` (12px bold, white, uppercase)
- Position: X=16, Y=20

#### MIX Slider
- Position: X=16, Y=44
- Size: 132×20px
- **Track:**
  - Rectangle: 132×20px
  - Fill: None
  - Stroke: 1px `border/soft` (#EAEAEA)
- **Thumb:**
  - Rectangle: ~32×20px (position based on value %)
  - Fill: `accent/slider-thumb` (#E8D348)
  - Stroke: 1px `border/thumb-outline` (#606060)

#### MIX Value Chip (optional in design)
- Text: "63%"
- Style: `Numeric/Readout`
- Position: Below slider, centered on thumb
- Add note: "Fades in on drag, fades out after 600ms"

#### EFFECT Button
- Position: X=280, Y=20
- Size: 64×32px
- **OFF State:**
  - Rectangle: 64×32px
  - Fill: `chassis/primary` (#3B7FC4)
  - Stroke: 2px `text/primary` (#FFFFFF)
  - Text: "EFFECT" (10px bold, white, centered)
- **Create variant for ON State:**
  - Fill: `accent/slider-thumb` (#E8D348)
  - Stroke: 2px `border/hard` (#111111)
  - Text: "EFFECT" (10px bold, black)

### Viewport Region (72-584px, centered with padding)

#### Viewport Container
- Rectangle: X=16, Y=88, W=328, H=480
- Fill: `viewport/base` (#000000)
- Stroke: 1px `border/hard` (#111111)

#### Waveform Traces (inside viewport)

**Input Signal Trace (CRT Green):**
- Line tool: 1px stroke
- Color: `accent/crt-green` (#3DD665)
- Draw a stepped/pixelated waveform along a horizontal midline
- Style: Use 48-64 vertical bars of varying heights
- Note: "This is a static mockup - actual waveform is dynamic"

**Processed Signal Trace (Voltage Lime):**
- Line tool: 1px stroke
- Color: `accent/voltage-lime` (#C3FF00)
- Draw another stepped waveform offset from the green line
- Should deviate above/below the green line

**Delta Fill:**
- Use the **Pen tool** to create a filled shape between the two traces
- Fill: `accent/voltage-lime` (#C3FF00)
- Opacity: 40-60% (adjust to taste)
- This represents "what FIELD adds"

**Alive Pulse Indicator:**
- Rectangle: 2×10px
- Fill: `accent/crt-green` (#3DD665)
- Position: Top-right corner of viewport (X=330, Y=92)
- Note: "Pulses at envelope rate - add animation prototype if desired"

### Footer Region (584-640px)

#### CHARACTER Label
- Text: "CHARACTER"
- Style: `Label/Primary` (12px bold, white, uppercase)
- Position: X=16, Y=592

#### CHARACTER Slider
- Position: X=80, Y=590
- Size: 264×22px (full width minus label, minus padding)
- **Track:**
  - Rectangle: 264×22px
  - Fill: Slightly darker blue or transparent
  - Stroke: 1px `border/soft` (#EAEAEA)
- **Thumb:**
  - Rectangle: ~48×22px (position based on value %)
  - Fill: `accent/slider-thumb` (#E8D348)
  - Stroke: 1px `border/thumb-outline` (#606060)

#### CHARACTER Value
- Text: "47%"
- Style: `Numeric/Readout`
- Position: X=310, Y=616 (bottom-right corner)

#### Tick Marks (optional)
- Small 1px vertical lines at 0, 25, 50, 75, 100% positions
- Color: `border/soft` (#EAEAEA) at 50% opacity

---

## Step 6: Create State Variations

### Interaction States

Create **Variants** or duplicate frames:

1. **Default State** (what you just built)
2. **EFFECT ON State:**
   - EFFECT button: yellow fill, black text
   - Viewport: Hide green input trace, show only lime trace + delta fill
3. **MIX Slider Dragging:**
   - Add 100ms flash effect (lime glow around thumb)
   - Show value chip below slider
4. **CHARACTER Slider Dragging:**
   - Add 100ms flash effect
   - Show value chip at bottom-right

---

## Step 7: Add Developer Annotations

Create a **Developer Handoff** page with the same design, but add **Annotation layers**:

### Critical Measurements

Add text boxes with arrows pointing to:
- "Header height: 72px"
- "Viewport padding: 16px on all sides"
- "Footer height: 56px"
- "Slider thumb has 1px grey outline (#606060) for pop"
- "Waveform: 48-64 samples horizontally, stepped (not smooth)"
- "All spacing uses 4px multiples"

### Color Callouts

Add color swatches with labels:
- "Chassis: #3B7FC4"
- "Yellow thumb: #E8D348 with #606060 outline"
- "Lime trace: #C3FF00"
- "Green trace: #3DD665"

### Typography Notes

- "All labels: DM Sans 12px Bold, UPPERCASE"
- "All values: IBM Plex Mono 12px Regular"

---

## Step 8: Export Assets (if needed)

If you have any static graphics or icons:
- Export as PNG @2x for high-DPI displays
- Or export as SVG for vector graphics in JUCE

---

## Step 9: Share with Claude for Implementation

Once your design is complete:

1. **Make the Figma file publicly viewable** (or share the link)
2. **Take screenshots** of:
   - Default state
   - EFFECT ON state
   - Developer annotations page
3. **Share with me** - I'll write pixel-perfect JUCE code

Or you can describe the design in detail, and I'll implement it exactly as you specify.

---

## Pro Tips

### Use Auto Layout
For buttons and sliders, use **Auto Layout** frames. This makes spacing adjustments instant.

### Prototyping (Optional)
Add simple prototype interactions:
- Click EFFECT button → Navigate to "EFFECT ON" frame
- This creates a clickable prototype for testing the feel

### Design System for Engine Suite
Once FIELD is finalized:
- Save components to a **shared library**
- When you design Pitch/Morph/Spectral, they'll use the same chassis/sliders
- Just swap the accent color per plugin

---

## What I Need from You After Figma

To implement in JUCE, I'll need:

1. **Figma link** (preferred) or **high-res screenshots**
2. **Confirmation of final measurements** (frame size, region heights)
3. **Any deviations from the design tokens** (if you tweaked colors)
4. **Priority:** Which state should I implement first (default, then EFFECT mode)

---

## Next Steps

1. Open Figma → Create new file
2. Follow this guide to set up variables and design FIELD
3. Share with me when ready
4. I'll write clean, production-ready JUCE code

**Estimated time:** 2-4 hours for a polished Figma design
**Payoff:** Pixel-perfect JUCE implementation with zero guesswork

Let me know when you're ready to share the design!
