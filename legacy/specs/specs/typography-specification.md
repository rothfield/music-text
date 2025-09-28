# Music-Text Typography Specification

## Overview

This specification defines typography standards for music-text notation display, based on analysis of DoReMiScript's proven CSS typography patterns. The goal is to achieve clear, collision-free musical notation rendering.

## Core Typography Principles

### Font Requirements
- **Primary Font**: Sans-serif for note symbols (`font-family: sans-serif`)
- **Syllable/Lyrics Font**: Serif for readability (`font-family: serif`)
- **UI Elements**: Sans-serif for controls and metadata

### Base Font Sizing
- **Root Size**: `14px` (scalable via `.doremiContent` wrapper)
- **Note Size**: `1.6em` relative to root (maintains proportional scaling)
- **Ornament Size**: `0.9em` for clarity without overwhelming main notation

## Critical Typography Challenges

### 1. Descender Collision Issues

**Problem**: Letters with descenders (g, j, p, q, y) conflict with:
- Beat group arcs positioned below content line
- Octave indicators and ornaments
- Syllable positioning

**Solutions**:
1. **Line Height Management**: `line-height: 10em` for notation lines (from DoReMiScript)
2. **Vertical Spacing**: Adequate padding between notation elements
3. **Collision Detection**: Dynamic positioning based on content analysis

### 2. Positioning System

**Absolute Positioning** (DoReMiScript pattern):
- **Octave Indicators**:
  - Upper: `top: 0.4em, left: 0.2em`
  - Lower: `bottom: 1.8em, left: 0.2em`
- **Ornaments**: `bottom: 0.98em` with `font-size: 0.9em`
- **Beat Arcs**: `bottom: -1.2em` with proper radius calculations

**Relative Positioning** for inline elements:
- Notes use `display: inline-block` with `position: relative`
- Margins: minimal (`0em`) to maintain tight spacing

### 3. Arc and Loop Typography

**Beat Group Arcs** (adapted from DoReMiScript):
```css
border-bottom: 0.1em solid;
border-bottom-left-radius: 0.8em;
border-bottom-right-radius: 0.8em;
padding-bottom: 0.75em;
```

**Chrome Border Fix**:
```css
border-left: 1px solid white;
border-right: 1px solid white;
```

## Responsive Typography

### Print Media
- **Smaller Arc Radius**: `2em` for better print clarity
- **Reduced Font Size**: `8px` root size
- **Border Adjustment**: `1px` borders for crisp printing

### High-DPI Displays
- Maintain relative sizing with `em` units
- Ensure arc clarity with appropriate border thickness

## Typography Standards

### Note Spacing
- **Standard Margin**: `0.0em` left/right for tight grouping
- **Barline Spacing**: `0.4em` left/right margins
- **Whitespace Handling**: Preserve semantic spacing without visual gaps

### Vertical Alignment
- **Baseline Alignment**: All notes on common baseline
- **Ornament Offset**: Consistent `0.98em` bottom positioning
- **Arc Clearance**: `1.2em` below baseline for collision avoidance

### Color and Contrast
- **Note Color**: High contrast for readability
- **Arc Color**: `#0366d6` (GitHub blue) for visual distinction
- **Background Handling**: Transparent or matching page background

## Implementation Guidelines

### CSS Architecture
1. **Modular Classes**: Separate concerns (positioning, typography, colors)
2. **Cascade Control**: Use specific selectors to avoid style conflicts
3. **Print Support**: Include `@media print` rules for optimal printing

### Performance Considerations
- **Font Loading**: Ensure fallback fonts for system compatibility
- **Rendering Optimization**: Use `transform` instead of positioning where possible
- **Memory Efficiency**: Avoid excessive DOM elements for visual effects

## Testing Requirements

### Cross-Browser Compatibility
- **Chrome**: Test border rendering with hairline fixes
- **Firefox**: Verify arc radius calculations
- **Safari**: Check font rendering and spacing consistency

### Content Scenarios
- **Mixed Content**: Notes with syllables and ornaments
- **Long Beat Groups**: 6+ note sequences with proper arc spanning
- **Descender Cases**: Test g, j, p, q, y in various notation contexts
- **Edge Cases**: Empty beat groups, overlapping ornaments

### Mordent Typography (Based on DoReMiScript)

**Symbol Rendering**:
- **Primary Symbol**: Unicode wavy line `∿` with 1.2x horizontal scaling
- **Alternative**: Traditional tilde `~` for compatibility
- **Inverted**: Right angle `⌐` for inverted mordents

**Positioning** (DoReMiScript pattern):
```css
.cm-music-mordent {
    position: absolute;
    font-size: 1.5em;
    top: 0.4em;
    left: 0.13em;
    z-index: 2;
    font-family: "Times New Roman", Georgia, serif;
}
```

**Visual Design**:
- **Color**: Purple (`#6f42c1`) matching ornament theme
- **Typography**: Serif font for classical appearance
- **Scale**: 1.5x larger than base text for visibility
- **Positioning**: Above and slightly left of target note

## Future Considerations

### Enhanced Typography Features
- **Ligature Support**: For connected note sequences
- **Variable Fonts**: For dynamic weight adjustments
- **Custom Font Loading**: Music-specific typefaces
- **Accessibility**: High contrast and screen reader support
- **Unicode Symbols**: Proper musical ornament characters (`∿`, `𝆝`)

### Performance Optimizations
- **CSS Grid**: For complex notation layouts
- **GPU Acceleration**: For smooth arc rendering
- **Lazy Loading**: For large notation documents

## References

- DoReMiScript CSS patterns (`resources/public/css/doremi.css`)
- Music notation typography standards
- Web typography best practices
- Cross-browser CSS compatibility guidelines