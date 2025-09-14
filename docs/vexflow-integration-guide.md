# VexFlow Integration Guide for Music-Text

*Curated documentation for VexFlow integration patterns specific to the music-text notation system*

## Overview

This guide focuses on VexFlow integration patterns actually used in the music-text project, providing practical examples and project-specific implementations rather than comprehensive API coverage.

**Library Version**: VexFlow 4.x  
**Integration**: Rust backend generates VexFlow-compatible JSON → JavaScript frontend renders  
**Output**: SVG/Canvas with 0.8 scaling factor for optimal visual density

## Core Architecture

### Rendering Pipeline
```
Music-Text Document → Rust AST → VexFlow JSON → JavaScript Renderer → SVG/Canvas
```

### Data Flow
```rust
// Rust: Document structure
pub struct VexFlowRenderer {
    pub fn render_data_from_document(&self, document: &Document) -> serde_json::Value
}

// Generated JSON structure
{
    "staves": [{
        "notes": [...],
        "key_signature": "C",
    }],
    "title": "Song Title",
    "author": "Composer Name"
}
```

## Library Loading & Initialization

### Dynamic Loading Pattern
```javascript
let vexflowLoaded = false;

async function loadVexFlow() {
    if (vexflowLoaded) return true;
    
    try {
        // Check if already available
        if (window.Vex && window.Vex.Flow) {
            vexflowLoaded = true;
            return true;
        }
        
        const script = document.createElement('script');
        script.src = 'assets/vexflow4.js';
        script.async = true;
        
        return new Promise((resolve, reject) => {
            script.onload = () => {
                if (window.Vex && window.Vex.Flow) {
                    vexflowLoaded = true;
                    resolve(true);
                } else {
                    reject(new Error('VexFlow loaded but not accessible'));
                }
            };
            script.onerror = () => reject(new Error('Failed to load VexFlow'));
            document.head.appendChild(script);
        });
    } catch (error) {
        console.error('VexFlow loading error:', error);
        return false;
    }
}
```

### Availability Check
```javascript
function isVexFlowLoaded() {
    return vexflowLoaded && window.Vex && window.Vex.Flow;
}
```

## Core VexFlow APIs

### Essential Imports
```javascript
const { 
    Renderer, Stave, Formatter, Voice, Beam, 
    StaveNote, Tuplet, Curve, StaveTie, 
    Ornament, Annotation, Dot, Accidental 
} = Vex.Flow;
```

### Renderer Setup
```javascript
// SVG renderer with scaling
const renderer = new Renderer(container, Renderer.Backends.SVG);
renderer.resize(canvasWidth, canvasHeight);
const context = renderer.getContext();

// Apply consistent scaling (project standard: 0.9)
context.scale(0.9, 0.9);
```

### Stave Creation
```javascript
// Create stave with full available width
const stave = new Stave(10, currentY, canvasWidth - 20);

// Add clef and key signature on first stave only
if (staveIndex === 0) {
    stave.addClef('treble');
    if (staveData.key_signature) {
        stave.addKeySignature(staveData.key_signature);
    }
}

stave.setContext(context);
stave.draw();
```

### Voice and Formatting
```javascript
// Create voice with flexible timing
const voice = new Voice({
    num_beats: 4,
    beat_value: 4,
    resolution: Vex.Flow.RESOLUTION
}).setStrict(false);

voice.addTickables(notes);

// Apply automatic accidental tracking
const keySignature = staveData.key_signature || 'C';
Vex.Flow.Accidental.applyAccidentals([voice], keySignature);

// Format with calculated minimum width
const formatter = new Formatter().joinVoices([voice]);
let minWidth = formatter.preCalculateMinTotalWidth([voice]);

// Handle VexFlow calculation bugs
if (isNaN(minWidth) || minWidth <= 0) {
    minWidth = notes.length * 50 + 100; // Fallback calculation
}

formatter.format([voice], minWidth);
voice.draw(context, stave);
```

## Music-Text Specific Patterns

### Multi-Notation System Support

The music-text system supports multiple pitch notation systems that all convert to VexFlow's standard pitch representation:

```javascript
// Music-text notation → VexFlow keys conversion examples:
// Number: "1" → "c/4", "7" → "b/4"  
// Sargam: "S" → "c/4", "N" → "b/4"
// ABC: "C" → "c/4", "B" → "b/4"
// DoReMi: "d" → "c/4", "t" → "b/4"
// Hindi: "स" → "c/4", "न" → "b/4"

// Example note creation with keys array
const note = new StaveNote({
    clef: 'treble',
    keys: element.keys || ['c/4'], // Always VexFlow format
    duration: element.duration || 'q'
});
```

### Octave Markers
Music-text spatial octave markers (dots above/below) are pre-processed into the appropriate VexFlow octave:

```javascript
// Music-text: Upper dot over "1" → VexFlow: "c/5"
// Music-text: Lower dot under "1" → VexFlow: "c/3"  
// Music-text: Double colon over "1" → VexFlow: "c/6"

// Keys are calculated during Rust processing:
{
    "keys": ["c/5"], // Already includes octave adjustment
    "duration": "q"
}
```

### Note Creation with Full Features
```javascript
function createAdvancedVexFlowNote(element) {
    const { StaveNote, Dot, Ornament, Accidental } = Vex.Flow;
    
    const note = new StaveNote({
        clef: 'treble',
        keys: element.keys || ['c/4'],
        duration: element.duration || 'q'
    });
    
    // Add dots (important for music-text rhythm representation)
    for (let i = 0; i < (element.dots || 0); i++) {
        if (note.addDot) {
            note.addDot(); // Preferred method
        } else {
            note.addModifier(new Dot(), 0); // Fallback
        }
    }
    
    // Add accidentals
    if (element.accidentals && element.accidentals.length > 0) {
        element.accidentals.forEach(acc => {
            if (acc.accidental) {
                note.addModifier(new Accidental(acc.accidental), acc.index);
            }
        });
    }
    
    // Add ornaments
    if (element.ornaments && element.ornaments.length > 0) {
        element.ornaments.forEach(ornamentType => {
            let vexflowType;
            switch (ornamentType) {
                case 'Mordent': vexflowType = 'mordent'; break;
                case 'Trill': vexflowType = 'trill'; break;
                case 'Turn': vexflowType = 'turn'; break;
            }
            
            if (vexflowType) {
                const ornament = new Ornament(vexflowType);
                note.addModifier(ornament, 0);
            }
        });
    }
    
    // Store syllable for manual positioning (see syllable section)
    if (element.syl && element.syl.trim()) {
        note._syllable = element.syl;
    }
    
    return note;
}
```

### Advanced Features

#### Tuplet Handling with Power-of-2 Algorithm
Music-text uses a specific power-of-2 algorithm for tuplet denominators:

```javascript
// Power-of-2 calculation for tuplet ratios
function getNextPowerOf2(n) {
    if (n <= 1) return 1;
    
    let power = 1;
    while (power < n) {
        power *= 2;
    }
    
    // Return the largest power of 2 that is less than n
    return power === n ? power : power / 2;
}

// Tuplet creation with calculated ratio
const tupletRatio = element.ratio || [element.divisions, getNextPowerOf2(element.divisions)];

const tuplet = new Tuplet(tupletNotes, {
    notes_occupied: tupletRatio[1],  // denominator (space of)
    num_notes: tupletRatio[0],       // numerator (actual notes)
    bracketed: true
});
tuplet.setContext(context);
```

#### Slurs vs Ties
Music-text distinguishes between slurs (different pitches) and ties (same pitch):

```javascript
// Slur creation (legato phrasing)
if (slurStartNote && endNote && slurStartNote !== endNote) {
    const slur = new Curve(slurStartNote, endNote, {
        cps: [{ x: 0, y: 10 }, { x: 0, y: 10 }] // Control points
    });
    slur.setContext(context);
}

// Tie creation (duration extension)
if (element.tied && notes.length >= 2) {
    const tie = new StaveTie({
        first_note: prevNote,
        last_note: currNote,
        first_indices: [0],
        last_indices: [0]
    });
    tie.setContext(context);
}
```

#### Automatic Beaming
```javascript
function shouldBeamTupletNotes(notes) {
    if (notes.length < 2) return false;
    
    return notes.every(note => {
        const duration = note.getDuration();
        return duration === '8' || duration === '16' || duration === '32' || duration === '64';
    });
}

// Beam creation
if (shouldBeamTupletNotes(notes)) {
    const beam = new Beam(notes);
    beam.setContext(context);
}
```

#### Barline Mapping
```javascript
function mapBarlineType(barType) {
    const { Barline } = Vex.Flow;
    switch (barType) {
        case 'repeat-begin': return Barline.type.REPEAT_BEGIN;
        case 'repeat-end': return Barline.type.REPEAT_END;
        case 'double': return Barline.type.DOUBLE;
        case 'final': return Barline.type.END;
        case 'double-repeat': return Barline.type.REPEAT_BOTH;
        case 'single':
        default: return Barline.type.SINGLE;
    }
}

// Apply barlines to stave
function processBarlines(stave, elements) {
    const firstElement = elements[0];
    if (firstElement?.type === 'BarLine') {
        const beginBarType = mapBarlineType(firstElement.bar_type);
        if (beginBarType) stave.setBegBarType(beginBarType);
    }
    
    const lastElement = elements[elements.length - 1];
    if (lastElement?.type === 'BarLine') {
        const endBarType = mapBarlineType(lastElement.bar_type);
        if (endBarType) stave.setEndBarType(endBarType);
    }
}
```

## Performance & Architecture

### Responsive Width Calculation
```javascript
// Calculate width based on content and viewport
let minWidth = 400; // Conservative base
const totalNotes = vexflowData.staves?.reduce((sum, stave) => 
    sum + (stave.notes?.length || 0), 0) || 0;
minWidth = totalNotes * 50 + 100; // ~50px per note + margins

// Add extra width for syllables
const totalSyllables = vexflowData.staves?.reduce((sum, stave) => 
    sum + (stave.notes?.filter(n => n.syl)?.length || 0), 0) || 0;
if (totalSyllables > 0) {
    minWidth += totalSyllables * 30; // 30px extra per syllable
}

// Use nearly full viewport width
const viewportWidth = window.innerWidth || document.documentElement.clientWidth || 800;
const canvasWidth = Math.max(viewportWidth - 20, minWidth + 80);
```

### Memory Management
```javascript
// Clean container before each render
container.innerHTML = '';

// VexFlow handles cleanup internally, but ensure context is properly scaled
const context = renderer.getContext();
context.scale(0.9, 0.9);
```

### Error Handling
```javascript
// Formatter width calculation safety check
let formatterMinWidth = formatter.preCalculateMinTotalWidth([voice]);

// Handle VexFlow calculation bugs
if (isNaN(formatterMinWidth) || formatterMinWidth <= 0) {
    formatterMinWidth = notes.length * 50 + 100; // Conservative fallback
}
```

## Project-Specific Patterns

### Title/Author Same-Line Layout
```javascript
// Render title and author on same line (music-text standard)
if (vexflowData.title || vexflowData.author) {
    context.save();
    
    // Title centered
    if (vexflowData.title) {
        context.setFont('serif', 18, 'bold');
        context.setFillStyle('#333');
        const titleWidth = context.measureText(vexflowData.title).width || vexflowData.title.length * 10;
        const titleX = (canvasWidth / 2) - (titleWidth / 2);
        context.fillText(vexflowData.title, titleX, currentY);
    }
    
    // Author right-aligned on same line
    if (vexflowData.author) {
        context.setFont('serif', 14, 'normal');
        context.setFillStyle('#666');
        const authorWidth = context.measureText(vexflowData.author).width || vexflowData.author.length * 8;
        const authorX = canvasWidth - authorWidth - 20;
        context.fillText(vexflowData.author, authorX, currentY);
    }
    
    context.restore();
    currentY += 35; // Space after title/author line
}
```

### Custom Syllable Positioning
Music-text uses custom syllable positioning relative to staff bottom instead of VexFlow's built-in lyric system:

```javascript
function drawSyllablesRelativeToStave(context, stave, notes) {
    const notesWithSyllables = notes.filter(note => note._syllable);
    if (notesWithSyllables.length === 0) return;
    
    // Calculate syllable Y position relative to staff bottom
    let maxY = stave.getYForLine(4) + 10; // Staff bottom + margin
    
    // Check note extents (stems, beams, etc.)
    notes.forEach(note => {
        if (note.getBoundingBox) {
            const bbox = note.getBoundingBox();
            maxY = Math.max(maxY, bbox.y + bbox.h + 5);
        }
    });
    
    const syllableY = maxY + 20; // Extra space for syllables
    
    // Draw syllables positioned under their notes
    notesWithSyllables.forEach(note => {
        if (note.getAbsoluteX && note._syllable) {
            const noteX = note.getAbsoluteX();
            
            context.save();
            context.font = 'italic 0.8em Arial';
            context.textAlign = 'center';
            context.fillStyle = '#000';
            context.fillText(note._syllable, noteX, syllableY);
            context.restore();
        }
    });
}
```

### Dotted Notes Width Adjustment
Music-text adds extra spacing for dotted notes during formatting:

```javascript
// Count dots and add extra space
const totalDots = staveData.notes.reduce((sum, note) => sum + (note.dots || 0), 0);
const dotExtraWidth = totalDots * 15; // 15px extra per dot

// Add to formatter width calculation
formatterMinWidth += dotExtraWidth;
formatter.format([voice], formatterMinWidth);
```

## Integration Examples

### Complete Rendering Function
```javascript
async function renderVexFlowNotation(vexflowData, containerId = 'vexflow-output') {
    // Ensure VexFlow is loaded
    if (!isVexFlowLoaded()) {
        const loaded = await loadVexFlow();
        if (!loaded) {
            console.error('Failed to load VexFlow');
            return false;
        }
    }
    
    const container = document.getElementById(containerId);
    if (!container) {
        console.error('VexFlow container not found:', containerId);
        return false;
    }
    
    try {
        container.innerHTML = '';
        
        const { Renderer, Stave, Formatter, Voice, Beam, StaveNote, Tuplet, Curve, StaveTie } = Vex.Flow;
        
        // Setup renderer with responsive width
        const renderer = new Renderer(container, Renderer.Backends.SVG);
        const canvasWidth = calculateResponsiveWidth(vexflowData);
        const canvasHeight = Math.max(200, (vexflowData.staves?.length || 1) * 150);
        
        renderer.resize(canvasWidth, canvasHeight);
        const context = renderer.getContext();
        context.scale(0.9, 0.9);
        
        let currentY = 30;
        
        // Render title/author if present
        currentY = renderTitleAuthor(context, vexflowData, canvasWidth, currentY);
        
        // Process each stave
        const staves = vexflowData.staves || [{ notes: [], key_signature: 'C' }];
        for (let staveIndex = 0; staveIndex < staves.length; staveIndex++) {
            currentY = await renderStave(context, staves[staveIndex], staveIndex, canvasWidth, currentY);
        }
        
        return true;
        
    } catch (error) {
        console.error('VexFlow rendering error:', error);
        container.innerHTML = `<div class="alert alert-danger">Rendering error: ${error.message}</div>`;
        return false;
    }
}
```

### Common Gotchas

#### VexFlow Version Compatibility
```javascript
// Check for method availability (varies between VexFlow versions)
if (note.addDot) {
    note.addDot(); // VexFlow 4.x preferred method
} else {
    note.addModifier(new Dot(), 0); // Fallback for older versions
}
```

#### Context State Management
```javascript
// Always save/restore context when changing settings
context.save();
context.setFont('serif', 18, 'bold');
context.setFillStyle('#333');
context.fillText(title, x, y);
context.restore(); // Critical: restore original context state
```

#### Formatter Edge Cases
```javascript
// Handle formatter calculation failures
let minWidth = formatter.preCalculateMinTotalWidth([voice]);
if (isNaN(minWidth) || minWidth <= 0) {
    // VexFlow sometimes returns NaN or 0, use fallback calculation
    minWidth = notes.length * 50 + 100;
    console.warn('Using fallback width calculation');
}
```

## Resources

- **VexFlow Official Docs**: https://vexflow.com/
- **VexFlow GitHub**: https://github.com/0xfe/vexflow
- **Music-Text Project Specs**: `/specs/vexflow-rendering-specification.md`
- **Working Implementation**: `/webapp/public/vexflow-renderer.js`

---

*This guide focuses on patterns actually used in the music-text project. For comprehensive VexFlow API documentation, refer to the official VexFlow documentation.*