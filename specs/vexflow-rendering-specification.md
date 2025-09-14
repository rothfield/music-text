# VexFlow Rendering Specification

## Overview

This specification defines the requirements for VexFlow-based music notation rendering in the Music Text system, covering musical element support, visual quality standards, performance requirements, and integration patterns.

## Rendering Architecture

### VexFlow Integration
- **Library Version**: VexFlow 4.x served from `/webapp/public/assets/vexflow4.js`
- **Dynamic Loading**: Asynchronous library loading with fallback handling
- **Scaling Factor**: Consistent 0.8 scaling for optimal visual density
- **Canvas/SVG**: Support both canvas and SVG output modes

### Rendering Pipeline
```
Music Text AST → VexFlow JSON → VexFlow Library → SVG/Canvas Output
```

## Musical Element Support

### Core Elements (Required)

#### Notes and Pitches
- **Pitch Systems**: Number (1-7), Sargam (S-N), ABC (A-G), DoReMi (d-t), Hindi Unicode
- **Accidentals**: Sharp (#), flat (b) with proper positioning
- **Octave Ranges**: Support multiple octaves with octave markers
- **Enharmonic Equivalents**: Consistent accidental rendering

#### Durations and Rhythm  
- **Note Values**: Whole, half, quarter, eighth, sixteenth notes
- **Dotted Notes**: Single and double dots
- **Rests**: Corresponding rest values for all note durations
- **Tied Notes**: Visual ties between notes of same pitch

#### Measures and Barlines
- **Standard Barlines**: Single `|`, double `||`
- **Repeat Barlines**: Start `|:`, end `:|`, final `|]`
- **Measure Spacing**: Consistent inter-measure spacing
- **Line Breaks**: Automatic staff wrapping for long passages

### Advanced Elements (Required)

#### Beaming
- **Automatic Beaming**: Eighth notes and smaller beamed by beat
- **Cross-beat Beaming**: Support beams across beat boundaries where appropriate
- **Nested Beaming**: Sixteenth note sub-beams within eighth note beams
- **Custom Beaming**: Override automatic beaming when specified

#### Tuplets
- **Common Ratios**: 3:2 (triplets), 5:4 (quintuplets), 7:4 (septuplets)
- **Power-of-2 Algorithm**: `getNextPowerOf2(n)` for tuplet calculation
- **Bracket Rendering**: Clear tuplet brackets with ratio numbers
- **Nested Tuplets**: Support tuplets within tuplets

#### Slurs and Ties
- **Slurs**: Curved lines connecting different pitches (legato)
- **Ties**: Curved lines connecting same pitches (duration extension)
- **Cross-system Slurs**: Slurs continuing across staff breaks
- **Control Points**: Proper curve control for natural appearance

### Extended Elements (Planned)

#### Ornaments
- **Grace Notes**: Acciaccatura and appoggiatura
- **Trills**: Trill symbols with optional accidentals
- **Mordents**: Upper and lower mordents
- **Turns**: Turn symbols with proper placement

#### Chord Symbols
- **Basic Chords**: Major, minor, diminished, augmented
- **Extended Chords**: 7th, 9th, 11th, 13th extensions
- **Positioning**: Above staff with proper alignment
- **Font Sizing**: Consistent chord symbol typography

#### Title and Metadata
- **Title Positioning**: Centered above first staff
- **Author/Composer**: Same line as title, right-aligned
- **Layout**: "Title                    Author" format
- **Typography**: Larger font for title, smaller for author

## Visual Quality Standards

### Staff Rendering
- **Staff Lines**: 5-line staff with consistent spacing
- **Staff Width**: Dynamic width based on content length  
- **Margins**: Appropriate left/right margins for clefs and key signatures
- **Line Thickness**: Professional line weight (1-2px)

### Note Rendering
- **Note Head Shape**: Proper oval note heads at correct angles
- **Stem Length**: Standard stem lengths (3.5 spaces)
- **Stem Direction**: Up for notes below middle line, down for above
- **Accidental Spacing**: Proper spacing between accidentals and note heads

### Typography and Symbols
- **Musical Font**: Professional music font rendering
- **Symbol Alignment**: Precise vertical alignment of all symbols
- **Text Rendering**: Clear, readable text for chord symbols and lyrics
- **Unicode Support**: Proper rendering of Hindi/Devanagari characters

### Spacing and Layout
- **Proportional Spacing**: Notes spaced according to duration
- **Minimum Spacing**: Adequate space for readability
- **Alignment**: Vertical alignment across multiple staves
- **Margins**: Consistent margins and padding

## Performance Requirements

### Rendering Speed
- **Real-time Updates**: < 200ms for typical notation (4-8 measures)
- **Complex Notation**: < 500ms for notation with tuplets and slurs
- **Progressive Rendering**: Show partial results during long renders
- **Debounced Updates**: 300ms debounce for real-time input

### Memory Management
- **Canvas Cleanup**: Proper cleanup of VexFlow canvas contexts
- **Memory Bounds**: No memory leaks during extended use
- **Resource Pooling**: Reuse VexFlow objects where possible
- **Garbage Collection**: Minimal GC pressure during rendering

### Scalability
- **Large Scores**: Handle documents with 20+ measures
- **Multiple Staves**: Support multi-staff rendering
- **Concurrent Rendering**: Handle multiple render requests
- **Browser Compatibility**: Consistent performance across target browsers

## Error Handling and Fallbacks

### Rendering Errors
- **Graceful Degradation**: Show partial results when possible
- **Error Visualization**: Clear indication of rendering problems
- **Debug Information**: Detailed error information for troubleshooting
- **Fallback Display**: Text-based fallback for critical failures

### Unsupported Features
- **Feature Detection**: Detect unsupported musical elements
- **Alternative Rendering**: Simplified rendering for unsupported features
- **User Feedback**: Clear indication of unsupported elements
- **Progressive Enhancement**: Core features work, advanced features optional

### Browser Compatibility
- **Canvas Fallback**: SVG preferred, canvas as fallback
- **Font Fallback**: Web-safe fonts if music fonts unavailable
- **API Compatibility**: Handle VexFlow API changes gracefully
- **Performance Adaptation**: Adjust rendering quality for slower devices

## Integration Requirements

### API Interface
```javascript
// Rendering function signature
async function renderVexFlow(musicData, options) {
    // Returns: { success: boolean, svg?: string, error?: string }
}

// Options structure
const options = {
    width: number,           // Canvas/SVG width
    height: number,          // Canvas/SVG height
    scale: number,           // Scaling factor (default: 0.8)
    format: 'svg' | 'canvas' // Output format
};
```

### Data Format
```javascript
// VexFlow-compatible JSON structure
const vexflowData = {
    staves: [{
        clef: 'treble',
        key_signature: 'C',
        time_signature: '4/4',
        notes: [
            { keys: ['c/4'], duration: 'q' },
            { keys: ['d/4'], duration: 'q' }
        ],
        tuplets: [...],
        slurs: [...],
        ties: [...]
    }]
};
```

### Configuration Options
- **Default Settings**: Sensible defaults for all rendering options
- **User Overrides**: Allow customization of visual appearance
- **Responsive Settings**: Adapt to container size and device capabilities
- **Accessibility Options**: High contrast, large fonts, etc.

## Test Requirements

### Visual Regression Tests
- **Reference Images**: Golden master images for comparison
- **Cross-browser Testing**: Consistent rendering across browsers
- **Pixel-perfect Accuracy**: Detect minute rendering differences
- **Automated Testing**: CI/CD integration for render tests

### Functional Test Cases

#### Basic Notation
```javascript
// Test case: Simple melody
input: "|1 2 3 4|"
expected: Four quarter notes on staff with barlines
```

#### Title and Author
```javascript
// Test case: Title with author
input: "Amazing Grace\nAuthor: John Newton\n\n|1 2 3 4|"
expected: "Amazing Grace                    John Newton" above staff
```

#### Advanced Features
```javascript
// Test case: Tuplets with slurs  
input: "|{3}123 456|"
expected: Two triplets with slur over first triplet
```

#### Error Conditions
```javascript  
// Test case: Invalid notation
input: "|invalid notation|"
expected: Error message with fallback display
```

### Performance Benchmarks
- **Render Time**: Target times for various complexity levels
- **Memory Usage**: Maximum memory consumption limits
- **Stress Testing**: Performance under heavy load
- **Regression Testing**: Performance doesn't degrade over time

## Implementation Status

### Current Implementation (✅)
- [x] Basic VexFlow library integration
- [x] Simple note and rest rendering
- [x] Basic barline support
- [x] Staff rendering with proper spacing

### In Progress (🚧)
- [ ] Advanced tuplet rendering with power-of-2 algorithm
- [ ] Slur and tie rendering with proper curves
- [ ] Automatic beaming for eighth notes and smaller
- [ ] Cross-system slur support

### Planned (📋)
- [ ] Ornament rendering (grace notes, trills, mordents)
- [ ] Chord symbol positioning and rendering
- [ ] Multi-staff system support
- [ ] Advanced typography and Unicode support

## Quality Assurance

### Acceptance Criteria
- [ ] All basic musical elements render correctly
- [ ] Visual quality matches professional notation software
- [ ] Performance meets target response times
- [ ] Error handling provides meaningful feedback
- [ ] Cross-browser consistency achieved

### Review Process
- **Visual Review**: Designer/musician approval of rendered output
- **Technical Review**: Code review focusing on performance and maintainability
- **User Testing**: Feedback from musicians using the interface
- **Accessibility Review**: Screen reader and keyboard navigation testing

---

*This specification ensures high-quality, performant VexFlow rendering that meets professional music notation standards while maintaining excellent user experience.*