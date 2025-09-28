# Music-Text Stylesheet Specification

## Overview
This document specifies the CSS styling requirements for rendering music-text notation in web browsers, particularly for the CodeMirror editor integration.

## Design Principles
1. **Semantic Clarity**: Visual elements should clearly represent their musical meaning
2. **Non-Intrusive**: Decorative elements should not overwhelm the notation
3. **Monospace Alignment**: All positioning assumes monospace fonts where 1ch = 1 character width
4. **Accessibility**: Colors and styles should be distinguishable and readable

## Color Scheme

### Primary Colors
- **Notes**: `deepskyblue` - Primary musical content
- **Barlines**: `mediumorchid` - Structural markers
- **Beat Loops**: `orange` - Rhythmic grouping indicators
- **Octave Markers**: `red` - Pitch modifiers
- **Lyrics/Syllables**: `chocolate` - Text underlay
- **Whitespace**: `transparent` with `lightgray` background on hover
- **Unknown Elements**: `gray` - Unrecognized notation

### Semantic Highlighting
```css
.cm-music-note { color: deepskyblue; }
.cm-music-barline { color: mediumorchid; font-weight: bold; }
.cm-music-dash { color: green; }
.cm-music-unknown { color: gray; }
.cm-music-syllable { color: chocolate; font-weight: bold; }
```

## Beat Loop Indicators

### Design: Tea Saucer Style
Beat loops should appear as shallow, saucer-like curves beneath grouped notes, not deep bowls. The loops should slightly impinge on the character bounding box for tight visual integration.

```css
.cm-music-note.beat-start::after {
    content: '';
    position: absolute;
    top: calc(100% - 0.1em);  /* Slightly impinge on character bounding box */
    left: 0;
    width: calc(var(--beat-char-loop-length) * 1ch);
    height: 0.25em;  /* Shallow height for saucer effect */
    border: 2px solid orange;
    border-top: none;
    border-bottom-left-radius: 50% 100%;  /* Flatter elliptical curve */
    border-bottom-right-radius: 50% 100%;  /* Flatter elliptical curve */
    z-index: 10;
    pointer-events: none;
}
```

### Key Characteristics:
- **Position**: Uses `calc(100% - 0.1em)` to start 0.1em before the bottom edge of character bbox
- **Overlap Strategy**: Slightly impinges on character bounding box rather than hanging completely below
- **Height**: 0.25em - Very shallow to create tea saucer effect (not deep bowl)
- **Border Radius**: `50% 100%` - Creates flatter elliptical curve rather than circular arc
- **Visual Integration**: Tight connection between loops and grouped characters
- **Non-intrusive**: Shallow profile doesn't overwhelm the notation

### Design Evolution
The loop placement has evolved through several iterations:
1. **Deep bowls** (height: 0.6em, radius: 50%) - Too visually prominent
2. **Hanging below bbox** (top: 100% + gap) - Too disconnected from characters
3. **Tea saucer with gap** (margin-top: 0.2em) - Unnecessarily spaced
4. **Tea saucer at bbox edge** (top: 100%) - Still felt detached
5. **Final: Tea saucer impinging** (top: calc(100% - 0.1em)) - Optimal visual integration

The final approach creates a subtle underline effect that feels connected to the characters while maintaining the shallow "tea saucer" aesthetic.

## Octave Indicators

### Upper Octave Markers (Above Staff)
```css
.octave-marker-1::before {
    content: '•';  /* One dot for +1 octave */
    position: absolute;
    top: -0.9em;
}

.octave-marker-2::before {
    content: '••';  /* Two dots for +2 octaves */
}

.octave-marker-3::before {
    content: '•••';  /* Three dots for +3 octaves */
}
```

### Lower Octave Markers (Below Staff)
```css
.octave-marker--1::after {
    content: '•';  /* One dot for -1 octave */
    position: absolute;
    bottom: -0.9em;
}
```

## Tuplet Indicators

### Tuplet Numbers
Display tuplet count above the first note of tuplet group:

```css
.cm-music-note::before {
    content: var(--tuplet, "");
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    top: -0.9em;
    font-size: 0.7em;
    opacity: 0.8;
}
```

## Tooltips

### Interactive Note Information
```css
.cm-music-note[style*="--title"]:hover:after {
    content: var(--title);
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    white-space: nowrap;
    z-index: 100;
    pointer-events: none;
    font-size: 12px;
}
```

## Duration-Based Styling

### Note Duration Classes
```css
.duration-1-1 { /* Whole note */
    font-weight: bold;
    text-shadow: 0 0 2px currentColor;
}

.duration-1-2 { /* Half note */
    font-weight: 600;
}

.duration-1-4 { /* Quarter note */
    /* Default styling */
}

.duration-1-8 { /* Eighth note */
    opacity: 0.9;
}

.duration-1-16 { /* Sixteenth note */
    opacity: 0.8;
    font-size: 0.95em;
}
```

## Beat Division Styling

### Visual Beat Grouping
```css
.beat-1 { /* Single note in beat */
    letter-spacing: 0.1em;
}

.beat-2 { /* Two notes in beat */
    letter-spacing: 0.05em;
}

.beat-3, .beat-4 { /* Three or four notes */
    letter-spacing: normal;
}

.beat-5, .beat-6, .beat-7 { /* Tuplets */
    letter-spacing: -0.02em;
    font-style: italic;
}
```

## Slur Indicators

### Upper Line Slurs
```css
.slur-start {
    position: relative;
}

.slur-start::before {
    content: '⌒';
    position: absolute;
    top: -0.5em;
    left: 0;
    right: 0;
    text-align: center;
    opacity: 0.6;
}
```

## Responsive Design

### Mobile Adjustments
```css
@media (max-width: 768px) {
    .cm-music-note {
        font-size: 1.2em;  /* Larger for touch */
    }

    .cm-music-note.beat-start::after {
        border-width: 3px;  /* Thicker lines for visibility */
    }
}
```

### High DPI Displays
```css
@media (-webkit-min-device-pixel-ratio: 2), (min-resolution: 192dpi) {
    .cm-music-note.beat-start::after {
        border-width: 1.5px;  /* Thinner lines for crisp display */
    }
}
```

## Animation and Transitions

### Hover Effects
```css
.cm-music-note {
    transition: color 0.15s ease, background-color 0.15s ease;
}

.cm-music-note:hover {
    background-color: rgba(135, 206, 250, 0.1);  /* Light blue highlight */
    cursor: help;
}
```

### Playback Highlighting (Future)
```css
.cm-music-note.playing {
    animation: pulse 0.5s ease;
    background-color: rgba(255, 215, 0, 0.3);  /* Gold highlight */
}

@keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.1); }
}
```

## Accessibility

### High Contrast Mode
```css
@media (prefers-contrast: high) {
    .cm-music-note { color: blue; }
    .cm-music-barline { color: purple; }
    .cm-music-note.beat-start::after {
        border-color: black;
        border-width: 3px;
    }
}
```

### Reduced Motion
```css
@media (prefers-reduced-motion: reduce) {
    .cm-music-note {
        transition: none;
    }

    .cm-music-note.playing {
        animation: none;
        background-color: rgba(255, 215, 0, 0.5);
    }
}
```

## Print Styles

```css
@media print {
    .cm-music-note.beat-start::after {
        border-color: black;
        border-width: 1px;
    }

    .cm-music-note[style*="--title"]:hover:after {
        display: none;  /* Hide tooltips in print */
    }
}
```

## CSS Custom Properties (Variables)

### Required Variables Set by JavaScript
- `--beat-char-loop-length`: Number of characters in beat group
- `--tuplet`: Tuplet number to display (e.g., "5")
- `--title`: Tooltip text for note information
- `--duration`: Note duration as fraction string
- `--octave`: Octave offset from base
- `--original-pitch`: Original pitch notation
- `--notation-system`: System used (Sargam, Western, etc.)

### Theme Variables
```css
:root {
    --music-note-color: deepskyblue;
    --music-barline-color: mediumorchid;
    --music-loop-color: orange;
    --music-loop-height: 0.25em;  /* Tea saucer height */
    --music-loop-offset: -0.8em;
    --music-font-family: ui-monospace, monospace;
}
```

## Browser Compatibility

### Required Features
- CSS Custom Properties (CSS Variables)
- CSS Grid/Flexbox
- CSS Transforms
- CSS calc() function
- CSS ::before/::after pseudo-elements

### Supported Browsers
- Chrome 80+
- Firefox 75+
- Safari 13.1+
- Edge 80+

## Performance Considerations

1. **Minimize Reflows**: Use transforms instead of position changes
2. **Batch Updates**: Update multiple CSS variables in single operation
3. **Limit Pseudo-Elements**: Maximum 2 per element (::before, ::after)
4. **GPU Acceleration**: Use `will-change` for animated elements

## Testing Requirements

### Visual Regression Tests
- Beat loops render correctly at different character counts
- Tooltips appear on hover
- Colors meet WCAG contrast requirements
- Layout remains stable during dynamic updates

### Cross-Browser Testing
- Consistent rendering across supported browsers
- Graceful degradation for older browsers
- Print output maintains readability

## Future Enhancements

1. **Dark Mode Support**: Alternate color schemes
2. **User Customization**: Allow color/size preferences
3. **Advanced Animations**: Note entry/exit animations
4. **3D Effects**: Depth for multi-voice notation
5. **SVG Integration**: Combine with VexFlow rendering