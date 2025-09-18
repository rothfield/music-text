# Lyrics and Syllables GUI Specification

## Overview
This specification defines how lyrics and syllables should be rendered, edited, and interact with musical notation in the GUI. The system supports two modes: **Auto-Assignment Mode** (LilyPond-style automatic syllable-to-note assignment) and **Manual Edit Mode** (direct lyric line editing).

## Mode Toggle System

### Auto-Assignment Mode (Default)
- **Active**: Syllables from lyrics_line automatically assigned to notes based on slurring
- **Rendering**: Server-side CSS/styles applied (similar to octave markers)
- **UI State**: Lyrics line hidden, auto-assigned syllables visible below notes
- **Behavior**: LilyPond-style assignment algorithm respects slur groupings

### Manual Edit Mode
- **Active**: Direct editing of lyrics_line in source text
- **Rendering**: Raw lyrics_line visible and editable
- **UI State**: Auto-assigned syllables hidden, lyrics line shown
- **Behavior**: User controls syllable placement and hyphenation manually

### Mode Switching
- **Toggle Control**: GUI button/checkbox to switch between modes
- **State Persistence**: Mode choice saved per document
- **Transition**: Smooth animation between mode states

## Visual Rendering

### Auto-Assignment Mode Rendering
- **Positioning**: Syllables appear below notes, aligned horizontally with note heads
- **Spacing**: 8-12px below lowest staff line or note stem
- **Server Styling**: CSS applied server-side, similar to octave marker rendering
- **Multi-verse**: Multiple lyrics_lines create stacked syllable assignments

### Manual Edit Mode Rendering
- **Positioning**: Lyrics_line appears below staff as editable text
- **Format**: Raw text following grammar: `syllable = letter+ (letter | digit | "'" | "-")*`
- **Spacing**: Standard line spacing below content
- **Editing**: In-place text editing with syntax highlighting

### Typography (Both Modes)
- **Font**: Sans-serif for clarity (system default or user preference)
- **Size**: 10-12pt relative to staff height
- **Color**: Inherit from theme or distinct text color (#333 default)
- **Style**: Regular weight, italic option for verses

## Auto-Assignment Algorithm

### LilyPond-Style Assignment
- **Input**: Parsed lyrics_line syllables + note sequence with slur information
- **Assignment Logic**:
  1. Syllables assigned to notes in sequence
  2. Slurred notes share syllables (melisma behavior)
  3. Hyphens create syllable continuation across notes
  4. Underscores extend syllables across multiple notes
- **Slur Respect**: Notes within slur groupings (`____` in upper_line) treated as single syllable target

### Assignment Rules
- **One-to-One**: Default syllable-to-note assignment
- **Melisma**: Single syllable spans slurred note group
- **Hyphenation**: Multi-syllable words split across notes using `-`
- **Extension**: `_` characters extend syllable across multiple notes
- **Overflow**: Extra syllables assigned to final notes, extra notes remain unassigned

### Grammar Integration
- **Source Format**: `syllable = letter+ (letter | digit | "'" | "-")*`
- **Syllable Parsing**: Respects existing grammar rules from grammar-specification.md
- **Spatial Alignment**: Follows spatial relationship rules for lower_line elements

## Syllable Behavior (Auto Mode)

### Hyphenation Display
- **Source Hyphens**: `-` in lyrics_line creates visual syllable continuation
- **Rendering**: Centered dash between assigned syllables
- **Dynamic Spacing**: Extend/contract based on note spacing
- **Proximity**: Hide if syllables too close (< 3px gap)

### Melisma Rendering
- **Trigger**: Slurred notes or `_` extension characters
- **Visual**: Thin horizontal line at baseline
- **Span**: From syllable end to last note of group
- **Line Breaks**: Continues across system boundaries

### Multi-verse Auto-Assignment
- **Multiple lyrics_lines**: Each creates separate syllable assignment layer
- **Stacking**: Verses stack vertically below notes
- **Synchronization**: All verses follow same assignment algorithm
- **Verse Numbering**: Optional verse indicators at line start

## Interactive Editing

### Manual Edit Mode Interaction
- **Click to Edit**: Click on lyrics_line to enter text editing mode
- **Syntax Aware**: Highlight syllable boundaries, hyphens, extensions
- **Navigation**: Arrow keys for character movement, Tab for word jumping
- **Format Keys**:
  - Space: Word boundary
  - Hyphen (`-`): Syllable break within word
  - Underscore (`_`): Melisma extension marker

### Auto Mode Interaction (Limited)
- **View Only**: Auto-assigned syllables are read-only
- **Source Edit**: Must switch to Manual mode to edit lyrics_line
- **Live Update**: Changes to lyrics_line immediately update auto-assignment
- **Slur Integration**: Editing slur markings affects syllable assignment

### Copy/Paste Behavior
- **Manual Mode**: Support plain text paste with format preservation
- **Multi-line**: Detect multiple verses on paste
- **Format Conversion**: Auto-convert between different lyric formats
- **Syllable Preservation**: Maintain hyphenation and extension markers

## Synchronization & Data Flow

### Auto-Assignment Pipeline
```
lyrics_line (source) → syllable parser → assignment algorithm →
slur analysis → positioned syllables → CSS styling → DOM rendering
```

### Manual Mode Pipeline
```
lyrics_line (source) → syntax highlighting →
direct text editing → live preview → format validation
```

### Note-Syllable Binding (Auto Mode)
- **Dynamic Binding**: Syllables automatically rebound on note changes
- **Slur Dependency**: Assignment updates when slur markings change
- **Resilience**: Algorithm handles note insertion/deletion gracefully
- **Feedback**: Visual indicators for assignment conflicts

### State Synchronization
- **Bi-directional**: Changes in either mode update source lyrics_line
- **Real-time**: Auto mode updates immediately on source changes
- **Conflict Resolution**: Manual edits take precedence over auto-assignment
- **Undo/Redo**: Mode changes and edits fully reversible

## Accessibility

### Screen Reader Support
- Semantic HTML with ARIA labels
- Logical reading order: staff → notes → lyrics
- Announce verse numbers and syllable positions

### Keyboard Navigation
- Tab through syllables sequentially
- Arrow keys for character-level editing
- Shortcuts for common operations (hyphenation, melisma)

## Error Handling

### Common Issues
- **Orphaned syllables**: Highlight in warning color
- **Missing syllables**: Show placeholder or dotted outline
- **Overflow text**: Truncate with ellipsis, show tooltip
- **Encoding issues**: Support Unicode, handle special characters

### Validation
- Check syllable-note count matching
- Warn on unusual hyphenation patterns
- Validate melisma endpoints

## Performance Considerations

### Rendering Optimization
- Cache syllable positions until edit
- Batch DOM updates for multi-syllable changes
- Lazy load verses not currently visible
- Virtual scrolling for long scores

### Memory Management
- Store lyrics as compressed text
- Index syllable-note mappings efficiently
- Clean up orphaned bindings

## Export/Import

### Supported Formats
- MusicXML: Full lyric preservation
- MIDI: Lyrics as meta events
- PDF: Embedded as text layer
- Plain text: Verse-by-verse export

### Round-trip Fidelity
- Preserve hyphenation
- Maintain verse ordering
- Keep syllable-note bindings
- Retain formatting hints

## GUI Layout Integration

### Responsive Behavior
- Syllables reflow with staff width changes
- Automatic font scaling for zoom levels
- Minimize vertical space in compact view
- Full lyrics in expanded view

### Theme Support
- Inherit colors from active theme
- Support high contrast mode
- Dark mode compatible rendering
- Print-optimized styling option

## User Preferences

### Configurable Options
- Font family and size
- Vertical spacing
- Hyphen style (dash, dot, none)
- Verse numbering (on/off, style)
- Melisma line style
- Default language for syllabification

### Persistence
- Save preferences per document
- Global defaults with per-score override
- Export preferences with document