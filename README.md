# Notation Parser

A **spatial rhythmic notation** parser that converts text-based musical notation into LilyPond staff notation. This project represents the latest evolution of a 30-year exploration of using **space as time** in musical notation.

**This parser handles a pragmatic pen-and-paper notation system** - an older type of handwritten musical notation where spatial relationships naturally express musical structure. The architecture may prove useful for OCRing such traditional handwritten musical documents.

## Core Innovation: Spatial Rhythmic Notation

The fundamental insight is using **horizontal space to represent time duration**:

```
S--r  --g-  -m--
│   │  │    │
│   │  │    └─ m gets 3 time units  
│   │  └───── g gets 2 time units
│   └──────── r gets 1 time unit
└─────────── S gets 3 time units
```

This **spatial-to-temporal conversion** is the most complex part of the data model, requiring:

1. **Beat Detection**: Group pitches with their trailing dashes (`S--` = 1 pitch + 2 dashes = 3 divisions)
2. **Division Counting**: Track dash consumption to avoid double-counting
3. **Fractional Conversion**: Map divisions to musical fractions (3 divisions in 4-beat = 3/4 duration)
4. **LilyPond Duration Mapping**: Convert fractions to Western note values (1/4 → "4", 1/2 → "2")

## Multi-Notation System Support

Supports multiple notation systems with automatic detection:

- **Western**: C D E F G A B (with sharps/flats)
- **Indian Classical**: S r R g G m M P d D n N (sargam)
- **Numbers**: 1 2 3 4 5 6 7 (numbered notation)

Each system is normalized to a common `PitchCode` enum for consistent processing.

## Architecture

### Modular Pipeline
```
Raw Text → Lexer → Parser → Flattener → Grouper → Converter
          ↓       ↓        ↓          ↓         ↓
       Tokens   Nodes   Spatial    Beats    LilyPond
                        Relations
```

### Core Modules

- **`models/`**: Data structures (Document, Node, Token, Metadata)
- **`lexer/`**: Text tokenization, chunk parsing, metadata extraction  
- **`parser/`**: Spatial relationship flattening, beat grouping algorithms
- **`display/`**: ANSI colorization, visualization, legend generation
- **`pitch/`**: Musical pitch handling with notation system detection
- **`lilypond_converter/`**: Rhythm calculation and LilyPond generation

### Key Data Structures

**Node**: Hierarchical structure representing musical elements
```rust
struct Node {
    node_type: String,    // "PITCH", "BEAT", "LINE", "BARLINE"
    value: String,        // Original text value
    pitch_code: Option<PitchCode>,  // Normalized pitch
    octave: Option<i8>,   // Octave information
    divisions: usize,     // Rhythmic subdivisions
    nodes: Vec<Node>,     // Child nodes (spatial relationships)
}
```

**Document**: Complete parsed composition
```rust
struct Document {
    metadata: Metadata,   // Title, directives
    nodes: Vec<Node>,     // Hierarchical content
}
```

## Complex Rhythm Processing

The rhythm system handles intricate patterns through **beat division analysis**:

### Example: `S--r  --g-  -m--`

1. **Beat Detection**: Each character in a beat represents an equal subdivision.
   - Beat 1: `S--r` (4 divisions: `S` gets 3, `r` gets 1)
   - Beat 2: `--g-` (4 divisions: rest gets 2, `g` gets 2)
   - Beat 3: `-m--` (4 divisions: rest gets 1, `m` gets 3)

2. **Fractional Conversion**: Assuming one beat is a quarter note:
   - `S` is 3/4 of the beat (dotted eighth note).
   - `r` is 1/4 of the beat (sixteenth note).
   - In Beat 2, an eighth rest is followed by `g`, an eighth note.
   - In Beat 3, a sixteenth rest is followed by `m`, a dotted eighth note.

3. **LilyPond Output**:
   ```lilypond
   c8. d16 r8 e8 r16 f8.
   ```

## Historical Context

This project builds on **doremi-script**, a comprehensive notation system that supported:

- 5 complete notation systems (ABC, Sargam, Numbers, Hindi, Doremi)
- Advanced ornaments and articulations
- Chord symbols and tala markings
- Complex multi-stave compositions
- State machine-based processing

The current implementation focuses on the **core spatial rhythm innovation** with modern Rust performance and safety.

## Visualization Features

- **ANSI colorization** with CSS-style parsing
- **Beat element underlining** for rhythm visualization
- **Spatial layout preservation** in terminal output
- **Configurable color schemes** for different token types

## Usage

```bash
# Parse notation file
cargo run input.123

# View colorized output
cargo run input.123 | less -R

# Generate LilyPond
cargo run input.123 > output.ly
lilypond output.ly
```

## Sample Input

```
Title: Example Song
Key: C

S--r  R-g-  G--m  P---
ban-  su-   ri    plays
```

## Technical Innovation

This notation parser demonstrates several computer music innovations:

1. **Spatial-Temporal Mapping**: Using 2D text layout to encode temporal relationships
2. **Multi-System Normalization**: Unified processing of diverse notation systems  
3. **Hierarchical Beat Modeling**: Tree structures for complex rhythmic relationships
4. **Proportional Duration Calculation**: Mathematical conversion of space to time
5. **Round-Trip Preservation**: Maintaining original layout through processing pipeline

The **spatial rhythmic notation** concept places this work in the tradition of experimental music notation alongside Morton Feldman's proportional scores, Krzysztof Penderecki's timeline notation, and other 20th-century innovations in graphic notation.

## Related Work

- **doremi-script**: Previous implementation in Clojure with full web interface
- **Proportional notation**: LilyPond's spacing where note duration = horizontal space
- **Graphic scores**: Visual music notation using spatial relationships
- **Timeline notation**: Contemporary classical music using time-proportional layouts

This represents a unique approach to making **spatial rhythm notation** both human-readable and computationally precise.