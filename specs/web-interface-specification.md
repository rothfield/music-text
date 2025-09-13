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

## User Interface Design

### Layout Structure
```
┌─────────────────────────────────────────────────────────────┐
│ Control Bar: [Parse] [LilyPond] [Clear]        Status      │
├─────────────────────────────────────────────────────────────┤
│ Textarea: Music notation input (auto-saving)               │  
├─────────────────────────────────────────────────────────────┤
│ Tabs: [Preview] [LilyPond] [JSON] [SVG]                    │
├─────────────────────────────────────────────────────────────┤
│ Tab Content: Dynamic output display                        │
└─────────────────────────────────────────────────────────────┘
```

### Control Bar
- **Parse Button**: Manual parsing with status feedback, switches to Preview tab
- **LilyPond Button**: Generates SVG via unified endpoint, switches to SVG tab  
- **Clear Button**: Clears all content and localStorage
- **Status Area**: Real-time feedback (success/error/loading states)

### Input Area
- **Responsive textarea**: Resizable, monospace font, minimal height 80px
- **Real-time parsing**: 300ms debounced updates to VexFlow preview
- **Placeholder**: `Enter music notation like: |S R G M|`
- **Advanced Notation Support**: 
  - **Accidentals**: `1# 2b 3` for sharp/flat notes with fancy Unicode rendering (♯, ♭, ♮)
  - **Dotted Notes**: `1-- 2-- 3-` for proper dotted rhythm rendering
  - **Mixed Systems**: Support for Sargam (S R G M), Number (1 2 3 4), Western (C D E F), DoReMi (d r m f)

### Output Tabs
1. **Preview**: Real-time VexFlow rendering with advanced features
   - **Proper Dotted Note Spacing**: Fixed spacing calculations for dotted rhythms
   - **Advanced Beaming**: Sophisticated beam grouping and tuplet support
   - **Accidental Rendering**: Sharp (♯), flat (♭), and natural (♮) symbols
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

### VexFlow Integration  
- **Advanced Renderer**: Full beaming, tuplets, slurs, ties support
- **Local Assets**: VexFlow library served from `public/assets/vexflow4.js`
- **Sophisticated Features**: Uses webapp.bu VexFlow renderer with complete feature set
- **Real-time Updates**: Renders as user types for immediate feedback

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
- **Static Assets**: VexFlow library served from `public/assets/`

### File Structure  
```
public/
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
- **Static Serving**: Root serves from `public/` directory

## Future Enhancements

### Potential Features
- **Export Options**: Save LilyPond/SVG files locally
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