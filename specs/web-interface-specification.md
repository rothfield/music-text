# Music Text Web Interface Specification

## Overview

This specification defines the **completely redesigned** web interface for the Music Text notation parser, providing a minimal, efficient, and user-friendly interface for real-time music notation parsing and visualization.

### Major UI Redesign (Current State)
- **Complete Rebuild**: Interface rebuilt from scratch with modern, clean design
- **Consolidated Structure**: Single `index.html` replaces previous multi-file approach
- **Enhanced Features**: Improved VexFlow rendering with proper dotted note spacing
- **Advanced Notation Support**: Full accidental support (♯, ♭, ♮) and complex rhythmic patterns

## Architecture

### Unified API Endpoint
- **Single Parse Endpoint**: `/api/parse` handles all parsing with optional SVG generation
- **Flag-based Processing**: `?generate_svg=true` adds LilyPond SVG compilation
- **Atomic Operations**: All outputs (VexFlow, LilyPond source, JSON, SVG) from single parse

### Client-Server Communication
```
Real-time: /api/parse?input=|1 2 3| → VexFlow + LilyPond source + JSON
SVG Generation: /api/parse?input=|1 2 3|&generate_svg=true → + SVG content
```

### JavaScript/No-JavaScript Dual Mode
- **Same HTML file** serves both JavaScript and no-JS modes
- **Query parameter control**: `?nojs=true` disables JavaScript features
- **Progressive enhancement**: Full functionality without JavaScript
- **Lynx compatible**: Works in text-based browsers

## User Interface Design

### Layout Structure
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Control Bar: [Parse] [LilyPond] [Clear] │ Octave: [↓↓↓][↓↓][↓][↑][↑↑][↑↑↑] │ Status │
├─────────────────────────────────────────────────────────────────────────────┤
│ Textarea: Music notation input (auto-saving, with selection support)      │
├─────────────────────────────────────────────────────────────────────────────┤
│ Tabs: [Preview] [LilyPond] [JSON] [SVG]                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│ Tab Content: Dynamic output display                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Control Bar
- **Parse Button**: Manual parsing with status feedback, switches to Preview tab
- **LilyPond Button**: Generates SVG via unified endpoint, switches to SVG tab
- **Clear Button**: Clears all content and localStorage
- **File Operations Section**: Import/Export functionality
  - **Menu vs Buttons Decision**: See File Operations UI Design below
  - **Save Options**: Export to various formats
  - **Load Options**: Import music files
- **MIDI Playback Section**: Transport controls for audio playback
  - **Play Button (▶️)**: Start MIDI playback of parsed notation
  - **Pause Button (⏸️)**: Pause/resume playback
  - **Stop Button (⏹️)**: Stop playback and reset position
  - **Tempo Slider**: Adjust playback speed (40-208 BPM)
  - **Tempo Display**: Shows current BPM setting
- **Octave Adjustment Section**: Six buttons for modifying selected text octaves
- **Status Area**: Real-time feedback (success/error/loading states)

### Input Area
- **Responsive textarea**: Resizable, monospace font, minimal height 80px
- **Real-time parsing**: 300ms debounced updates to VexFlow preview
- **Placeholder**: `Enter music notation like: |S R G M|`

### Octave Adjustment Controls

#### Button Layout
```
┌─────────────────────────────────────────────────────────────┐
│ [Parse] [LilyPond] [Clear] │ Octave: [↓↓↓] [↓↓] [↓] [↑] [↑↑] [↑↑↑] │
└─────────────────────────────────────────────────────────────┘
```

#### Button Functions
- **↓↓↓ (Lowest)**: Adds `:` below selected notes (-2 octaves)
- **↓↓ (Lowish)**: Adds `.` below selected notes (-1 octave) [same as Lower for now]
- **↓ (Lower)**: Adds `.` below selected notes (-1 octave)
- **↑ (Higher)**: Adds `.` above selected notes (+1 octave)
- **↑↑ (Highish)**: Adds `.` above selected notes (+1 octave) [same as Higher for now]
- **↑↑↑ (Highest)**: Adds `:` above selected notes (+3 octaves)

#### Behavior
1. **Selection Required**: Buttons only work when text is selected in the textarea
2. **Note Detection**: Only modifies characters that are musical notes (S, R, G, M, P, D, N, 1-7, A-G, etc.)
3. **Spatial Addition**: Adds octave markers in appropriate spatial lines:
   - **Above notes**: Upper line (increases octave)
   - **Below notes**: Lower line (decreases octave)
4. **Focus and Selection Preservation**:
   - Restores focus to textarea after button click
   - Maintains text selection after modification
   - Preserves cursor position if no selection
5. **Real-time Update**: Triggers immediate re-parsing and preview update

#### Implementation Details

##### Text Processing Algorithm
```javascript
function applyOctaveAdjustment(selectedText, octaveType) {
  // 1. Split selection into lines
  const lines = selectedText.split('\n');

  // 2. Identify content line (contains notes)
  const contentLineIndex = findContentLine(lines);

  // 3. Create spatial structure if needed
  if (octaveType.includes('up')) {
    // Add upper line above content line
    lines.splice(contentLineIndex, 0, createUpperOctaveLine(lines[contentLineIndex], octaveType));
  } else {
    // Add lower line below content line
    lines.splice(contentLineIndex + 1, 0, createLowerOctaveLine(lines[contentLineIndex], octaveType));
  }

  // 4. Return modified text
  return lines.join('\n');
}

function handleOctaveButtonClick(octaveType) {
  const textarea = document.getElementById('input');

  // 1. Preserve selection state
  const selectionStart = textarea.selectionStart;
  const selectionEnd = textarea.selectionEnd;
  const selectedText = textarea.value.substring(selectionStart, selectionEnd);

  // 2. Apply octave modification
  const modifiedText = applyOctaveAdjustment(selectedText, octaveType);

  // 3. Replace selected text
  const beforeSelection = textarea.value.substring(0, selectionStart);
  const afterSelection = textarea.value.substring(selectionEnd);
  textarea.value = beforeSelection + modifiedText + afterSelection;

  // 4. Restore focus and selection
  textarea.focus();
  textarea.setSelectionRange(
    selectionStart,
    selectionStart + modifiedText.length
  );

  // 5. Trigger re-parsing
  parseInput();
}
```

##### Octave Marker Mapping
- **Lowest (↓↓↓)**: `:` symbol → -2 octaves
- **Lowish (↓↓)**: `.` symbol → -1 octave (same as Lower currently)
- **Lower (↓)**: `.` symbol → -1 octave
- **Higher (↑)**: `.` symbol → +1 octave
- **Highish (↑↑)**: `.` symbol → +1 octave (same as Higher currently)
- **Highest (↑↑↑)**: `:` symbol → +3 octaves

#### Visual Feedback
- **Button States**: Active/inactive based on text selection
- **Tooltips**: Show octave adjustment amount on hover
- **Keyboard Shortcuts**:
  - `Ctrl+Shift+↓` → Lower
  - `Ctrl+Shift+↑` → Higher
  - `Ctrl+Alt+Shift+↓` → Lowest
  - `Ctrl+Alt+Shift+↑` → Highest

#### Error Handling
- **No Selection**: Buttons disabled, tooltip shows "Select notes first"
- **No Notes in Selection**: Show warning "Selection contains no musical notes"
- **Complex Selection**: Handle multi-line selections intelligently
- **Existing Octave Markers**: Merge or replace existing markers appropriately

#### Usage Examples

##### Example 1: Adding Higher Octave
**Before** (user selects "R G"):
```
| S R G M |
```

**After** clicking ↑ (Higher):
```
    . .
| S R G M |
```

##### Example 2: Adding Lowest Octave
**Before** (user selects "S R"):
```
| S R G M |
```

**After** clicking ↓↓↓ (Lowest):
```
| S R G M |
  : :
```

##### Example 3: Complex Multi-line Selection
**Before** (user selects entire phrase):
```
    .
| S R G M |
  .
```

**After** clicking ↑↑↑ (Highest):
```
    : : : :
| S R G M |
  .
```

#### Future Extensions
- **Lowish (↓↓)**: Will use `*` symbol for -1.5 octaves (when implemented)
- **Highish (↑↑)**: Will use `*` symbol for +1.5 octaves (when implemented)
- **Batch Operations**: Select multiple phrases and apply octave adjustments
- **Undo/Redo**: Standard text editing operations for octave modifications

### Output Tabs
1. **Preview**: Real-time VexFlow rendering with advanced features
   - **Proper Dotted Note Spacing**: Fixed spacing calculations for dotted rhythms
   - **Advanced Beaming**: Sophisticated beam grouping and tuplet support
   - **Accidental Rendering**: Sharp (♯), flat (♭), and natural (♮) symbols
   - **Beat Loop Rendering**: Lower loops only drawn when 2 or more [pitch|dash] present in beat
     - Single pitch/dash elements: No loop styling applied
     - Multiple pitch/dash elements: Visual loop arc appears below beat group
     - Breath marks don't count toward 2+ requirement but included in loop width
   - **Pitch Typography**: Musical pitches displayed in bold while preserving monospace character width
     - Bold font weight for visual emphasis of musical notes
     - Consistent character width maintained using tabular numbers
     - Monospace font family enforcement to prevent layout shifts
     - Font synthesis disabled for performance and consistency
   - **Notation System Consistency**: Single notation system per document enforced
     - First content line determines notation system for entire document
     - Subsequent content lines must use the same notation system
     - Parser validates notation system consistency across staves
2. **LilyPond**: Formatted LilyPond source code with syntax highlighting
3. **JSON**: Raw parser output for debugging
4. **SVG**: High-quality LilyPond-generated SVG with professional typography

## Behavior Specifications

### Real-time Updates
- **Keystroke Processing**: 300ms debounced calls to `/api/parse`
- **Silent Updates**: No status messages during typing
- **VexFlow Priority**: Always updates Preview tab as user types
- **LilyPond Source**: Updates alongside VexFlow for immediate inspection

### Manual Actions
- **Parse Button**: Explicit parsing with status feedback, tab switching
- **LilyPond Button**: SVG generation with loading states, error handling
- **Tab Switching**: Preserves focus and cursor position in textarea

### Persistence Layer

#### Local Storage Keys
- `music-text-input`: User's notation text
- `music-text-cursor`: Cursor position `{start, end}`  
- `music-text-active-tab`: Last active tab name

#### Cursor Position Management
- **Automatic Saving**: On keyup, mouseup, click events
- **Pre-action Saving**: Before tab switches and button clicks
- **Restoration Events**: Tab switches, button actions, page reload
- **Boundary Safety**: Positions validated against current text length

#### Persistence Behavior
- **Continuous Saving**: Text and cursor position saved as user types
- **Session Restoration**: Page reload restores text, cursor, and active tab
- **Cross-session Persistence**: Data survives browser restart

## Technical Implementation

### Frontend Architecture
- **Vanilla JavaScript**: No framework dependencies
- **Event-driven**: DOM events trigger API calls and UI updates
- **Debounced Input**: Prevents excessive API calls during typing
- **Focus Management**: Maintains textarea focus across all interactions

### No-JavaScript Fallback
When `?nojs=true` is present:
- JavaScript checks query parameter and disables itself
- HTML forms provide all functionality
- Server returns full HTML pages instead of JSON
- File operations use standard form POST
- See [Retro UI Specification](retro-ui-specification.md) for details

### Typography Requirements
- **Monospace Preservation**: Bold pitches maintain consistent character width
  - `font-variant-numeric: tabular-nums` for uniform number spacing
  - `font-feature-settings: "tnum"` for OpenType tabular figures
  - `font-synthesis: none` to prevent artificial bold generation
  - Explicit monospace font stack for cross-platform consistency

### VexFlow Integration
- **Advanced Renderer**: Full beaming, tuplets, slurs, ties support
- **Local Assets**: VexFlow library served from `public/assets/vexflow4.js`
- **Sophisticated Features**: Uses webapp.bu VexFlow renderer with complete feature set
- **Real-time Updates**: Renders as user types for immediate feedback

### MIDI Playback Integration
- **Tone.js Library**: Web Audio API abstraction for high-quality audio synthesis
- **PitchCode Mapping**: Direct conversion from parser output to MIDI note numbers
- **Rhythm Processing**: Rational durations converted to time-based scheduling
- **Transport Controls**: Play/pause/stop with tempo adjustment (40-208 BPM)
- **Error Handling**: Graceful fallback when Web Audio API unavailable

### Error Handling
- **Graceful Degradation**: Parse errors show in relevant tabs
- **Debug Information**: Expandable debug details for troubleshooting  
- **Console Logging**: SVG generation errors logged server-side
- **Fallback Display**: Raw data shown when rendering fails

### Performance Optimizations
- **Single Parse Operation**: Unified endpoint eliminates duplicate processing
- **Debounced Updates**: 300ms delay prevents API spam
- **Lightweight UI**: Minimal CSS, no external frameworks
- **Asset Locality**: VexFlow served locally for faster loading

## User Experience Goals

### Immediate Feedback
- **Instant Visual**: VexFlow updates as user types
- **No Blocking**: Real-time updates don't show loading states
- **Source Inspection**: LilyPond source available immediately

### Consistent Context
- **Preserved Focus**: Cursor stays in textarea across all actions
- **Position Memory**: Cursor returns to exact typing position
- **Session Continuity**: Work resumes exactly where user left off

### Minimal Interface
- **Clean Design**: GitHub-inspired styling, professional appearance
- **Compact Layout**: Maximum screen space for content
- **Essential Controls**: Only necessary buttons and features
- **No Tooltips**: Self-explanatory interface elements

## API Specification

### Parse Endpoint
```
GET /api/parse?input={notation}&generate_svg={boolean}

Response:
{
  "success": boolean,
  "parsed_document": object,
  "processed_staves": object, 
  "detected_notation_systems": array,
  "lilypond": string,           // LilyPond source code
  "lilypond_svg": string,       // SVG content (if generate_svg=true)
  "vexflow": object,            // VexFlow rendering data
  "vexflow_svg": string,
  "error": string
}
```

### Processing Flow
1. **Input Validation**: Unicode character checking
2. **Notation Parsing**: Single `process_notation()` call
3. **Conditional SVG**: LilyPond compilation only if requested
4. **Response Assembly**: All outputs from single parse operation

## Quality Assurance

### Browser Compatibility
- **Modern Browsers**: Chrome, Firefox, Safari, Edge (recent versions)
- **JavaScript Features**: ES6+, RequestAnimationFrame, localStorage
- **CSS Features**: Flexbox, CSS Grid, modern selectors

### Performance Targets
- **Initial Load**: < 1 second to interactive
- **Parse Response**: < 200ms for typical notation
- **SVG Generation**: < 2 seconds for complex notation
- **Memory Usage**: Stable across extended sessions

### Accessibility
- **Keyboard Navigation**: Full keyboard accessibility
- **Focus Management**: Clear focus indicators
- **Screen Readers**: Semantic HTML structure
- **Color Contrast**: Professional color scheme with sufficient contrast

## Deployment Requirements

### Server Dependencies
- **LilyPond Binary**: Must be installed and accessible in PATH
- **Temp Directory**: Writable `/tmp/music-text-svg/` for SVG generation
- **Static Assets**: VexFlow library served from `webapp/public/assets/`

### File Structure  
```
webapp/
└── public/
    ├── index.html              # Redesigned main interface (single file)
    ├── vexflow-renderer.js     # Enhanced VexFlow renderer with dotted note fixes
    └── assets/
        └── vexflow4.js         # VexFlow library
```

### Current Implementation Highlights
- **Unified Codebase**: Consolidated from multiple HTML files to single clean interface
- **Enhanced Rendering**: VexFlow dotted note spacing fix (15px per dot) implemented
- **Fraction Unification**: Standardized fraction handling across LilyPond and VexFlow renderers
- **Unicode Support**: Professional accidental symbols (♯, ♭, ♮) in output rendering

### Environment Configuration
- **Server Port**: 3000 (configurable)
- **CORS**: Permissive for development
- **Static Serving**: Root serves from `webapp/public/` directory

## File Operations UI Design

### Design Approach Comparison

#### Option A: Dropdown Menu System
```
┌─────────────────────────────────────────┐
│ [File ▼] [Edit ▼] [View ▼] [Help ▼]    │
├─────────────────────────────────────────┤
│ File Menu:                              │
│ ├─ New                    Ctrl+N        │
│ ├─ Open...                Ctrl+O        │
│ ├─ Save                   Ctrl+S        │
│ ├─ Save As...             Ctrl+Shift+S  │
│ ├─ Export ►                             │
│ │  ├─ PDF (LilyPond)                    │
│ │  ├─ MIDI                              │
│ │  └─ SVG                               │
│ ├─ Import ►                             │
│ │  ├─ MusicXML                          │
│ │  ├─ MIDI                              │
│ │  └─ ABC                               │
│ └─ Exit                                 │
└─────────────────────────────────────────┘
```

**Advantages:**
- Familiar desktop application paradigm
- Organized hierarchical structure
- Keyboard shortcuts visible
- Reduces button clutter
- Professional appearance

**Disadvantages:**
- Extra clicks for common operations
- Mobile-unfriendly
- May feel heavy for web app

#### Option B: Button Bar System
```
┌─────────────────────────────────────────────────────────────┐
│ [📁 Open] [💾 Save] [📥 Export ▼] [📤 Import ▼] │ [Parse] ... │
└─────────────────────────────────────────────────────────────┘
```

**Advantages:**
- One-click access to common operations
- Visual icons for quick recognition
- Mobile-friendly touch targets
- Modern web application feel

**Disadvantages:**
- Limited space for all options
- Can become cluttered
- Less discoverable features

#### Recommended Hybrid Approach

**Primary Actions as Buttons + Secondary in Dropdown:**
```
┌────────────────────────────────────────────────────────────────┐
│ [Open] [Save] [Export ▼] │ [Parse] [Clear] │ MIDI Controls... │
└────────────────────────────────────────────────────────────────┘

Export dropdown:
├─ PDF (LilyPond)
├─ MIDI File
├─ LilyPond Source (.ly)
└─ Music-Text (.mt)
```

### File Operations Implementation

#### Save Operations
1. **Save Music-Text (.mt)**
   - Default format for the application
   - Preserves all spatial information
   - Plain text format

2. **Export PDF (via LilyPond)**
   - Server-side LilyPond compilation
   - High-quality engraving
   - Print-ready output

3. **Export MIDI**
   - Standard MIDI file format
   - Preserves tempo and dynamics
   - Compatible with all DAWs

4. **Export LilyPond Source (.ly)**
   - For manual editing in LilyPond
   - Includes generated comments
   - Professional typesetting

#### Load Operations
1. **Open Music-Text (.mt)**
   - Native format loading
   - Instant parsing and display

2. **Import from other formats**
   - See Import Specification below

## Future Enhancements

### Potential Features
- **Voice Input**: Speech-to-notation transcription
- **Notation Templates**: Quick-start templates for common patterns
- **Keyboard Shortcuts**: Power-user acceleration
- **Theme Options**: Light/dark mode support
- **Collaborative Features**: Share notation via URLs

### Scalability Considerations  
- **Caching Layer**: Redis for parse result caching
- **CDN Integration**: Static asset distribution
- **API Rate Limiting**: Prevent abuse in production
- **WebSocket Support**: Real-time collaborative editing

## Maintenance

### Monitoring
- **Error Tracking**: Client and server-side error logging
- **Performance Metrics**: Parse time and SVG generation duration
- **User Analytics**: Feature usage and interaction patterns

### Updates
- **VexFlow Updates**: Regular library updates for new features
- **Browser Compatibility**: Testing against new browser versions  
- **Security Patches**: Regular dependency updates

---

*This specification defines a production-ready web interface for music text notation parsing with emphasis on user experience, performance, and reliability.*