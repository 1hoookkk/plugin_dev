# FIELD — Industrial Digital Minimalism Style Guide

**Identity:** Intentional Voltage
**Philosophy:** Industrial digital minimalism — unapologetically bright, brutally flat, built for signal.

---

## Overview

FIELD is a morphing filter utility that feels like a live signal instrument. The design communicates precision, energy, and control through stark contrast between deep industrial blue and high-voltage lime. Every element should feel charged, alert, and deliberately minimal — as if the plugin itself runs at the edge of clipping.

**This is not:**
- Warm or nostalgic
- Retro computing kitsch
- Faux hardware mimicry

**This is:**
- Data-bright utility
- Modern industrial lab aesthetic
- Pure signal, motion, and clarity

---

## Color Palette

### Primary Colors

| Color | HEX | Usage | Notes |
|-------|-----|-------|-------|
| **Vibrant Blue** | `#3B7FC4` | Chassis, control backgrounds | Saturated, high-contrast blue. Main frame color. |
| **Viewport Black** | `#000000` | Visualization background | Infinite depth. Motion container. |
| **Voltage Lime** | `#C3FF00` | Processed signal trace, delta fill | Primary energy accent. Intentionally overbright. |
| **Warm Yellow** | `#E8D348` | Slider thumbs | Slightly desaturated yellow with grey outline for pop. |
| **CRT Green** | `#3DD665` | Input signal trace, alive pulse | Secondary energy accent. |

### Text Colors

| Color | HEX | Usage |
|-------|-----|-------|
| **White** | `#FFFFFF` | Primary labels on blue chassis |
| **Soft White** | `#EAEAEA` | Secondary text on black viewport |
| **Black** | `#000000` | Text on lime backgrounds (EFFECT ON state) |

### Border Colors

| Color | HEX | Usage |
|-------|-----|-------|
| **Hard Border** | `#111111` | 1px separation between blue and black regions |
| **Soft Border** | `#EAEAEA` | 1px light separation on dark backgrounds |
| **Thumb Outline** | `#606060` | Subtle grey outline around yellow slider thumbs (creates pop) |

### Design Principles

1. **No gradients** - Every color meets another cleanly with hard edges
2. **No alpha blending** - Solid colors only (except for delta fill opacity modulation)
3. **Vibrating contrast** - Blue vs. lime should feel electrically charged
4. **1px borders only** - Clean separation, no thick strokes

---

## Typography

### Font Families

| Type | Font Stack | Usage |
|------|-----------|-------|
| **Labels** | DM Sans, Satoshi, -apple-system, sans-serif | All control labels (MIX, CHARACTER, EFFECT) |
| **Numerics** | IBM Plex Mono, 'Courier New', monospace | Value readouts, percentage chips |

### Typography Scale

| Element | Size | Weight | Transform | Example |
|---------|------|--------|-----------|---------|
| **Control Labels** | 12px | Bold (700) | UPPERCASE | `MIX`, `CHARACTER`, `EFFECT` |
| **Value Readouts** | 12px | Regular (400) | None | `63%`, `47%` |
| **Captions** | 11px | Regular (400) | None | "EFFECT ONLY" indicator |

### Typography Rules

- **All labels are uppercase** - Machine annotation style
- **Monospace for data** - Numbers should feel technical and precise
- **No playful language** - Functional tone only
- **White on blue, soft white on black** - Maximize legibility

---

## Spacing & Layout

### Base Grid System

- **Base unit:** 4px
- **All spacing uses multiples of 4px:** 4px, 8px, 12px, 16px, 24px
- **No arbitrary values** - Maintains engineered rhythm

### Standard Spacing Values

| Token | Value | Usage |
|-------|-------|-------|
| `spacing.unit` | 4px | Minimum spacing increment |
| `spacing.control-padding` | 16px | Padding around control regions |
| `spacing.viewport-margin` | 0px | Viewport bleeds to edges with 1px border |

### Layout Dimensions

| Region | Height | Notes |
|--------|--------|-------|
| **Header Rail** | 72px | MIX slider (left) + EFFECT button (right) |
| **Viewport** | Flexible | Fills remaining space between header and footer |
| **Footer Rail** | 56px | CHARACTER slider (full width) |

### Aspect Ratio

- **Default:** 9:16 portrait (360×640 px)
- **Minimum:** 320×568 px
- **Scales proportionally** - Maintains portrait orientation

---

## UI Elements

### Borders & Corners

| Property | Value | Rationale |
|----------|-------|-----------|
| **Border Width** | 1px exactly | Clean, precise separation |
| **Border Radius** | 0px (sharp corners) | No rounded edges - pure industrial geometry |
| **Border Style** | Solid | No dashes, dots, or fancy strokes |

**Visual Rule:** When blue meets black, they meet with a single 1px line. No soft transitions.

### Buttons

#### EFFECT Button

**OFF State:**
- Background: `#2E5E92` (Industrial Blue)
- Text: `#FFFFFF` (White)
- Border: 1px `#111111`

**ON State:**
- Background: `#C3FF00` (Voltage Lime)
- Text: `#000000` (Black)
- Border: 1px `#111111`

**Interaction:**
- 100ms lime flash on click
- Binary toggle (ON/OFF latch)
- Momentary when held

**Feel:** Like a power switch - unmissable and definitive.

### Sliders

#### MIX Slider (Top-Left)

- **Dimensions:** 132×20 px
- **Track:** Transparent (no fill)
- **Thumb:** Warm yellow (`#E8D348`) with subtle grey outline (`#606060`)
- **Value chip:** Fades in on adjust, shows percentage (`63%`), fades out after 600ms
- **Flash:** 100ms lime flash on drag start
- **Note:** The grey outline makes the yellow thumb pop against the blue background

#### CHARACTER Slider (Bottom Rail)

- **Dimensions:** Full width minus 16px padding × 22px height
- **Track:** Dark fill (matches blue chassis)
- **Thumb:** Warm yellow (`#E8D348`) with subtle grey outline (`#606060`)
- **Ticks:** Only at 0, 25, 50, 75, 100
- **Role:** Hero control - main gesture for FIELD's personality
- **Note:** The yellow with outline creates visual hierarchy as the primary control

### Visualizer (Twin Traces Δ-Fill)

**Components:**

1. **Input Line:**
   - 1px solid
   - Color: `#3DD665` (CRT Green)
   - Position: Stable midline

2. **Processed Line:**
   - 1px solid
   - Color: `#C3FF00` (Voltage Lime)
   - Position: Moves above/below input based on processing

3. **Delta Fill:**
   - Solid lime between input and processed lines
   - Represents "what FIELD adds"
   - Opacity modulated by MIX parameter

**Behavior:**

- **Normal mode:** Both traces visible with delta fill
- **EFFECT ON:** Input line hidden (only delta remains)
- **MIX:** Controls delta fill opacity
- **CHARACTER:** Increases vertical separation and complexity

**Visual Style:**

- 48-64 step samples horizontally
- Digital and rhythmic (not analog/fluid)
- No blur or anti-aliasing
- 1px borders frame like an oscilloscope window

---

## Animation & Feedback

### Interaction Timing

| Event | Duration | Easing | Purpose |
|-------|----------|--------|---------|
| **Press/Drag Flash** | 100ms | Linear | Instant certainty of response |
| **Slider Movement** | 150ms | Ease-out | Smooth but fast value change |
| **Value Chip Fade** | 600ms | Linear | Unobtrusive readout dismiss |

### Motion Principles

1. **Feels fast** - All interactions respond within one frame
2. **No waiting** - Visualizer predicts state while audio updates
3. **Electric responsiveness** - Like touching live voltage
4. **Binary feedback** - Instant flash confirms every action

### Alive Pulse

- **Indicator:** 2×10 px green dash at top of visualizer
- **Color:** `#3DD665` (CRT Green)
- **Behavior:** Pulses at envelope follower rate
- **Purpose:** Shows system is actively processing signal

---

## Component States

### Interactive States

| State | Visual Change | Duration |
|-------|--------------|----------|
| **Default** | Base colors | - |
| **Hover** | No change (not applicable for plugin UI) | - |
| **Active/Dragging** | 100ms lime flash, then base color | 100ms |
| **Disabled** | N/A (all controls always active) | - |

### Feedback Flash Pattern

**On any interaction (click, drag start):**
1. Element flashes to `#C3FF00` (Voltage Lime) for 100ms
2. Returns to base state
3. Value updates simultaneously

**Purpose:** Creates electrical "snap" feeling - instant tactile feedback.

---

## Accessibility & Contrast

### Color Contrast Ratios

| Combination | Ratio | WCAG Level | Usage |
|-------------|-------|------------|-------|
| White on Industrial Blue | 4.8:1 | AA | Primary labels |
| Black on Voltage Lime | 14.1:1 | AAA | EFFECT button ON state |
| CRT Green on Black | 8.2:1 | AAA | Input trace |
| Voltage Lime on Black | 15.1:1 | AAA | Processed trace |

All text combinations meet WCAG AA standards minimum.

### Visual Clarity

- 1px traces are intentionally high-contrast against pure black
- No reliance on color alone - lime flash provides motion feedback
- Voltage lime is deliberately overbright for maximum visibility

---

## Design System Philosophy

### "Intentional Voltage" Aesthetic

**Core metaphor:** The plugin itself is a piece of electronics under load

**Visual language:**
- Sharp edges (no curves)
- Flat colors (no gradients)
- High contrast (vibrating color relationships)
- Minimal elements (only what's necessary)

**Emotional tone:**
- Charged, not calm
- Alert, not friendly
- Precise, not warm
- Industrial, not nostalgic

### What FIELD Is NOT

❌ **VGA nostalgia** - This isn't retro computing kitsch
❌ **Analog warmth** - No faux hardware skeuomorphism
❌ **Friendly UI** - No rounded corners or soft shadows
❌ **Data visualization** - Not about pretty graphs, about raw signal

### What FIELD IS

✅ **Live signal instrument** - Feels electrically alive
✅ **Modern industrial lab** - Clean, engineered, intentional
✅ **Pure utility** - Built for work, not for show
✅ **Voltage aesthetic** - Running at 99% capacity, always

---

## Engine Suite System

FIELD establishes the design language for all **Engine Suite** plugins.

### Shared Design DNA

Every Engine plugin shares:
- Deep blue chassis (`#2E5E92`)
- Black viewport (`#000000`)
- Overbright accent (unique per module)
- White calm text (`#FFFFFF`)
- Flat geometry / 1px edges
- 9:16 portrait ratio

### Plugin Differentiation

Each module gets its own accent color:
- **FIELD:** Voltage Lime (`#C3FF00`)
- **PITCH:** [TBD - different accent]
- **MORPH:** [TBD - different accent]
- **SPECTRAL:** [TBD - different accent]

This creates visual family unity while allowing instant recognition.

---

## Implementation Checklist

When building FIELD components, verify:

- [ ] All colors come from `FIELD_DESIGN_TOKENS.json`
- [ ] No border-radius applied (sharp corners only)
- [ ] All spacing uses 4px multiples
- [ ] Text is uppercase for labels, monospace for numbers
- [ ] Borders are exactly 1px solid
- [ ] Flash animations are 100ms duration
- [ ] Slider movements use 150ms ease-out
- [ ] Background is pure black `#000000` or industrial blue `#2E5E92`
- [ ] Voltage lime is used sparingly for maximum impact
- [ ] No gradients, glows, or soft shadows
- [ ] Interaction feels instant and electric

---

## Success Criteria

**FIELD is successful when it feels like a live signal, not a product UI.**

At a glance, FIELD should communicate:
- ✅ Data is moving
- ✅ The system is awake
- ✅ The user has control

**No clutter, no ornamentation, no nostalgia.**
**Just raw signal, motion, and clarity.**

---

**Built for sound. Built for speed. Built for signal.**
