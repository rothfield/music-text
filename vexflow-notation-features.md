# VexFlow Notation Features Guide

A comprehensive guide to using VexFlow for advanced musical notation display including barlines, slurs, lyrics, key signatures, and accidentals.

## Table of Contents
- [Barlines](#barlines)
- [Slurs and Curves](#slurs-and-curves)
- [Lyrics and Text Annotations](#lyrics-and-text-annotations)
- [Key Signatures](#key-signatures)
- [Accidentals](#accidentals)
- [Integration Examples](#integration-examples)

---

## Barlines

VexFlow provides comprehensive barline support through two main classes: `BarNote` and `Barline`.

### Barline Types

```javascript
// Available barline types
Vex.Flow.Barline.type = {
  SINGLE: 1,      // Single barline |
  DOUBLE: 2,      // Double barline ||
  END: 3,         // End barline |]
  REPEAT_BEGIN: 4, // Repeat begin |:
  REPEAT_END: 5,   // Repeat end :|
  REPEAT_BOTH: 6,  // Double repeat :||:
  NONE: 7         // No barline
};
```

### Creating Barlines

#### Method 1: Using BarNote (Recommended for voices)
```javascript
// Create a bar note that can be added to a voice
const barNote = new Vex.Flow.BarNote(Vex.Flow.Barline.type.DOUBLE);

// Add to voice (BarNotes have no duration and consume no ticks)
voice.addTickables([note1, note2, barNote, note3, note4]);
```

#### Method 2: Using Stave Barlines
```javascript
// Add barlines directly to staves
const stave = new Vex.Flow.Stave(10, 40, 400);
stave.setBegBarType(Vex.Flow.Barline.type.REPEAT_BEGIN);
stave.setEndBarType(Vex.Flow.Barline.type.REPEAT_END);
```

### VexTab Notation
```javascript
// VexTab shorthand for barlines
"=||"  // Double Bar
"=|:"  // Repeat Begin  
"=:|"  // Repeat End
"=::"  // Double Repeat
"=|="  // End Bar
```

### Multiple Measures Example
```javascript
// Create multiple measures on one stave
const voice = new Vex.Flow.Voice({num_beats: 8, beat_value: 4});
voice.addTickables([
  note1, note2, note3, note4,
  new Vex.Flow.BarNote(),  // Measure separator
  note5, note6, note7, note8
]);
```

---

## Slurs and Curves

VexFlow implements slurs through the `Curve` class, which renders curved lines between notes.

### Basic Curve Creation
```javascript
// Create a curve (slur) between two notes
const curve = new Vex.Flow.Curve(startNote, endNote, options);
curve.setContext(context).draw();
```

### Configuration Options
```javascript
const curveOptions = {
  spacing: 2,           // Distance from notes
  thickness: 2,         // Line thickness
  x_shift: 0,          // Horizontal offset
  y_shift: 10,         // Vertical offset  
  position: Vex.Flow.Curve.Position.NEAR_HEAD, // Position relative to note
  invert: false,       // Flip curve direction
  cps: [               // Control points for curve shape
    { x: 0, y: 10 }, 
    { x: 0, y: 10 }
  ]
};

const slur = new Vex.Flow.Curve(note1, note4, curveOptions);
```

### Advanced Slur Positioning
```javascript
// Position slurs above or below notes
const upperSlur = new Vex.Flow.Curve(note1, note2, {
  position: Vex.Flow.Curve.Position.NEAR_TOP,
  y_shift: -20,
  cps: [{ x: 0, y: -15 }, { x: 0, y: -15 }]
});

const lowerSlur = new Vex.Flow.Curve(note3, note4, {
  position: Vex.Flow.Curve.Position.NEAR_HEAD,
  y_shift: 25,
  cps: [{ x: 0, y: 20 }, { x: 0, y: 20 }]
});
```

### Partial Slurs (Cross-measure)
```javascript
// Slurs that span across measures (missing start or end note)
const partialSlur = new Vex.Flow.Curve(null, endNote, options); // Start from previous measure
const partialSlur2 = new Vex.Flow.Curve(startNote, null, options); // Continue to next measure
```

---

## Lyrics and Text Annotations

VexFlow provides multiple approaches for adding lyrics and text to notation.

### Method 1: Annotation Class (Most Common)
```javascript
// Create a note with lyrics
const note = new Vex.Flow.StaveNote({keys: ['c/4'], duration: 'q'});

// Add lyrics below the note
const lyrics = new Vex.Flow.Annotation('Do');
lyrics.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);
lyrics.setFont('Arial', 12, 'normal');
note.addModifier(lyrics);

// Add chord symbols above
const chord = new Vex.Flow.Annotation('Cmaj7');
chord.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.TOP);
chord.setFont('Arial', 10, 'bold');
note.addModifier(chord);
```

### Annotation Positioning Options
```javascript
// Horizontal justification
Vex.Flow.Annotation.Justify = {
  LEFT,           // Left-align text
  CENTER,         // Center text
  RIGHT,          // Right-align text  
  CENTER_STEM     // Center on stem
};

// Vertical justification  
Vex.Flow.Annotation.VerticalJustify = {
  TOP,            // Above staff
  CENTER,         // Center of staff
  BOTTOM,         // Below staff
  CENTER_STEM     // Center on stem
};
```

### Method 2: VexTab Annotation Syntax
```javascript
// VexTab notation with lyrics
const vexTab = `
notes :q 5/5 5/4 5/3 ^3^ $Do,Re,Mi,Fa$ :h 4/4 $.top.$ $Cmaj7$
options font-size=12
`;

// Position control
"$.top.$"     // Position above staff
"$.bottom.$"  // Position below staff  
"$.medium.$"  // Medium font size
"$.big.$"     // Large font size
"$.italic.$"  // Italic style
```

### Method 3: TextNote Class (For Dynamics, Chords)
```javascript
// Create text positioned in time
const textNote = new Vex.Flow.TextNote({
  text: 'mp',                    // Text content
  duration: 'q',                 // Duration alignment
  line: -1,                      // Staff line position
  smooth: true,                  // Smooth positioning
  ignore_ticks: false           // Whether to consume ticks
});

// Add to voice like regular notes
voice.addTickables([note1, textNote, note2]);
```

### Multi-line Lyrics Example
```javascript
// Create multiple verses
const verse1 = new Vex.Flow.Annotation('Twin-');
verse1.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);

const verse2 = new Vex.Flow.Annotation('Star-');
verse2.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);
verse2.setFont('Arial', 12, 'italic');

note.addModifier(verse1);
note.addModifier(verse2);
```

---

## Key Signatures

VexFlow supports both standard and custom key signatures through the `KeySignature` class.

### Standard Key Signatures
```javascript
// Add key signature to stave
const stave = new Vex.Flow.Stave(10, 40, 400);
stave.addKeySignature('G');    // G major (1 sharp)
stave.addKeySignature('F');    // F major (1 flat)
stave.addKeySignature('D');    // D major (2 sharps)
stave.addKeySignature('Bb');   // Bb major (2 flats)
```

### Custom Key Signatures
```javascript
// Create custom key signature with specific accidentals
const keySignature = new Vex.Flow.KeySignature(
  'C',                    // Base key
  null,                   // Cancel key (optional)
  ['b', '#', 'n']        // Alter key specs
);

// Add to stave
stave.addModifier(keySignature);
```

### Key Signature Parameters
```javascript
// Full constructor signature
new Vex.Flow.KeySignature(keySpec, cancelKeySpec, alterKeySpec)

// Examples:
new Vex.Flow.KeySignature('G');           // G major
new Vex.Flow.KeySignature('C', 'G');      // Cancel G major, show C major  
new Vex.Flow.KeySignature('C', null, ['b', '#']); // Custom accidentals
```

### Clef-Aware Positioning
```javascript
// Key signatures automatically adjust for different clefs
const trebleStave = new Vex.Flow.Stave(10, 40, 200);
trebleStave.addClef('treble').addKeySignature('G');

const bassStave = new Vex.Flow.Stave(10, 140, 200);  
bassStave.addClef('bass').addKeySignature('G');

// Accidentals appear in correct positions for each clef
```

### Microtonal Key Signatures
```javascript
// Support for non-Western key signatures
const arabicKey = new Vex.Flow.KeySignature('C', null, ['b', 'h', '#']);
// Where 'h' represents half-flat accidental
```

---

## Accidentals

VexFlow provides comprehensive accidental support with automatic application and manual control.

### Basic Accidental Types
```javascript
// Standard accidental codes
'#'   // Sharp
'##'  // Double sharp  
'b'   // Flat
'bb'  // Double flat
'n'   // Natural
'@'   // Flat (alternative)
'@@'  // Double flat (alternative)
```

### Manual Accidental Creation
```javascript
// Add accidental to specific note
const note = new Vex.Flow.StaveNote({keys: ['c/4'], duration: 'q'});
const accidental = new Vex.Flow.Accidental('#');
note.addModifier(accidental, 0); // Add to first key
```

### Automatic Accidental Application
```javascript
// Automatically apply accidentals based on key signature
const voices = [voice1, voice2, voice3];
const keySignature = 'G'; // G major

// VexFlow will automatically add necessary accidentals
Vex.Flow.Accidental.applyAccidentals(voices, keySignature);
```

### Multiple Accidentals per Note
```javascript
// For chords with different accidentals
const chord = new Vex.Flow.StaveNote({
  keys: ['c/4', 'e/4', 'g/4'], 
  duration: 'q'
});

chord.addModifier(new Vex.Flow.Accidental('#'), 0);  // C sharp
chord.addModifier(new Vex.Flow.Accidental('b'), 1);  // E flat  
chord.addModifier(new Vex.Flow.Accidental('n'), 2);  // G natural
```

### Accidental Positioning
```javascript
// Control accidental placement
const accidental = new Vex.Flow.Accidental('#');
accidental.setXShift(-5);  // Move left
accidental.setYShift(2);   // Move up

note.addModifier(accidental, 0);
```

---

## Integration Examples

### Complete Song Notation Example
```javascript
function createSongNotation() {
  // Setup
  const div = document.getElementById('notation');
  const renderer = new Vex.Flow.Renderer(div, Vex.Flow.Renderer.Backends.SVG);
  renderer.resize(800, 300);
  const context = renderer.getContext();
  
  // Create stave with key signature and time signature
  const stave = new Vex.Flow.Stave(10, 40, 750);
  stave.addClef('treble')
       .addKeySignature('G')
       .addTimeSignature('4/4');
  stave.setContext(context).draw();
  
  // Create notes with lyrics and accidentals
  const notes = [
    new Vex.Flow.StaveNote({keys: ['g/4'], duration: 'q'}),
    new Vex.Flow.StaveNote({keys: ['a/4'], duration: 'q'}),
    new Vex.Flow.StaveNote({keys: ['b/4'], duration: 'q'}),
    new Vex.Flow.StaveNote({keys: ['c/5'], duration: 'q'})
  ];
  
  // Add lyrics
  const lyrics = ['Do', 'Re', 'Mi', 'Fa'];
  notes.forEach((note, i) => {
    const lyric = new Vex.Flow.Annotation(lyrics[i]);
    lyric.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);
    note.addModifier(lyric);
  });
  
  // Add accidental to third note
  notes[2].addModifier(new Vex.Flow.Accidental('#'), 0);
  
  // Create voice and add barline
  const voice = new Vex.Flow.Voice({num_beats: 4, beat_value: 4});
  voice.addTickables([...notes, new Vex.Flow.BarNote()]);
  
  // Add slur
  const slur = new Vex.Flow.Curve(notes[0], notes[3], {
    y_shift: -20,
    cps: [{x: 0, y: -15}, {x: 0, y: -15}]
  });
  
  // Format and draw
  const formatter = new Vex.Flow.Formatter();
  formatter.joinVoices([voice]).format([voice], 700);
  
  voice.draw(context, stave);
  slur.setContext(context).draw();
}
```

### Chord Chart with Lyrics
```javascript
function createChordChart() {
  // Setup stave
  const stave = new Vex.Flow.Stave(10, 40, 600);
  stave.addClef('treble').addKeySignature('C');
  
  // Create chord progression
  const chords = [
    new Vex.Flow.StaveNote({keys: ['c/4', 'e/4', 'g/4'], duration: 'w'}),
    new Vex.Flow.StaveNote({keys: ['f/4', 'a/4', 'c/5'], duration: 'w'}),
    new Vex.Flow.StaveNote({keys: ['g/4', 'b/4', 'd/5'], duration: 'w'}),
    new Vex.Flow.StaveNote({keys: ['c/4', 'e/4', 'g/4'], duration: 'w'})
  ];
  
  // Add chord symbols above
  const chordNames = ['C', 'F', 'G', 'C'];
  chords.forEach((chord, i) => {
    const symbol = new Vex.Flow.Annotation(chordNames[i]);
    symbol.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.TOP);
    symbol.setFont('Arial', 14, 'bold');
    chord.addModifier(symbol);
  });
  
  // Add lyrics below  
  const lyrics = ['A-', 'ma-', 'zing', 'Grace'];
  chords.forEach((chord, i) => {
    const lyric = new Vex.Flow.Annotation(lyrics[i]);
    lyric.setVerticalJustification(Vex.Flow.Annotation.VerticalJustify.BOTTOM);
    chord.addModifier(lyric);
  });
}
```

### Multi-measure with Repeats
```javascript
function createRepeatingSection() {
  // First measure
  const stave1 = new Vex.Flow.Stave(10, 40, 200);
  stave1.addClef('treble')
        .addKeySignature('G')
        .setBegBarType(Vex.Flow.Barline.type.REPEAT_BEGIN);
  
  // Second measure  
  const stave2 = new Vex.Flow.Stave(210, 40, 200);
  stave2.setEndBarType(Vex.Flow.Barline.type.REPEAT_END);
  
  // Or use BarNote approach for single stave
  const voice = new Vex.Flow.Voice({num_beats: 8, beat_value: 4});
  voice.addTickables([
    new Vex.Flow.BarNote(Vex.Flow.Barline.type.REPEAT_BEGIN),
    ...measure1Notes,
    new Vex.Flow.BarNote(),
    ...measure2Notes,
    new Vex.Flow.BarNote(Vex.Flow.Barline.type.REPEAT_END)
  ]);
}
```

### Best Practices

1. **Performance**: Use `Accidental.applyAccidentals()` for automatic accidental management rather than manual application.

2. **Layout**: Create separate staves for complex multi-measure notation rather than cramming everything into one stave.

3. **Text Positioning**: Use consistent vertical justification for lyrics and consistent fonts throughout.

4. **Slur Curves**: Adjust control points (`cps`) based on the distance between notes for natural-looking curves.

5. **Barlines**: Use `BarNote` for barlines within voices, use stave barlines for measure boundaries.

6. **Key Changes**: Create new staves for key signature changes rather than trying to modify existing ones.

This guide covers the essential VexFlow features for creating rich musical notation. For more advanced features and detailed API documentation, refer to the VexFlow test files and official documentation.