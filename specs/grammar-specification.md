# Music Text Grammar Specification

## Notation System Rules

**Single Notation System Per Document**: Each document can only contain one notation system, determined by the first content line encountered. All subsequent content lines must use the same notation system.

Supported notation systems:
- **Sargam**: S, R, G, M, P, D, N (with accidentals)
- **Number**: 1, 2, 3, 4, 5, 6, 7 (with accidentals)
- **Western**: A, B, C, D, E, F, G (with accidentals)
- **Bhatkhande**: स, र, ग, म, प, ध, न
- **Tabla**: dha, dhin, ta, ka, etc.

## Overview

This specification defines the formal grammar for the music-text notation language, supporting multiple pitch notation systems with spatial rhythm representation and hierarchical document structure.

## Grammar Rules (EBNF)

### Document Structure

```ebnf
document = blank_lines* document_element (blank_lines* document_element)* blank_lines*

document_element = header_block | stave | single_content_line

header_block = header_content

header_content = header_line (newline header_line)*

header_line = title_line | directive_line | text_line

single_content_line = content_line

directive_line = directive whitespace* (newline | EOI)

directive = key ":" value

title_line = whitespace{3,} title whitespace{3,} author whitespace* (newline | EOI)

title = text_content

author = text_content

text_line = text_content (newline | EOI)


key = identifier

value = text_content

text_content = (!newline ANY)*

identifier = letter (letter | digit | "_")*

document_body = stave (blank_lines stave)*

stave = upper_line* content_line (lower_line | lyrics_line)* (blank_lines | (whitespace* newline)* EOI)

blank_lines = newline (whitespace* newline)+
newline = "\n"
whitespace = " "
letter = "A".."Z" | "a".."z"
digit = "0".."9"
```

### Content Lines

```ebnf
content_line = line_number? non-beat-element* beat (non-beat-elemnt | beat)  newline
non-beat-element = barline | whitespace
beat = spatially-delimited-beat |
      (pitch | dash) beat-element*
beat-element = pitch | dash | breath-mark   // breath-marks can appear anywhere within a beat (middle or end)

pitch = note_in_system

note_in_system = sargam_note | number_note | western_note | tabla_note | hindi_note
dash = "-"
breath-mark = "'"   // apostrophe/tick mark indicates a breath or pause, can appear anywhere within a beat
```

## Design Decision: No Measures

We decided not to support measure grouping for simplicity (KISS principle). Content lines contain beats directly. Beats are maximal sequences of beat elements that terminate when encountering:
- End of line (EOL)
- End of input (EOI)
- Non-beat elements (spaces, barlines, etc.)

## Design Decision: Atomic Pitches

Pitches are treated as atomic units (e.g., "1", "1#", "1b", "S", "S#") rather than decomposed into base pitch + accidental components.

### Annotation Lines

```ebnf
annotation_line = upper_line | lower_line | lyrics_line

upper_line = upper_line_element+ (newline | EOI)
lower_line = lower_line_element+ (newline | EOI)
lyrics_line = syllable+ (newline | EOI)
```

### Upper Line Elements

```ebnf
upper_line_element = octave_marker | slur_indicator | ornament | chord | mordent | tala | space | unknown_upper

octave_marker = "." | ":"   // "." = single, ":" = double (interpretation depends on position)
slur_indicator = "_" "_"+   // exactly two or more consecutive underscores for slur marking (minimum length: 2)
ornament = "<" pitch+ ">" | pitch+
chord = "[" chord_symbol "]"
mordent = "~"
tala = "+" | "0" | digit
space = " "+
unknown_upper = !octave_marker !("_" "_") !space !ornament !chord !mordent !tala ANY+
```

### Lower Line Elements

```ebnf
lower_line_element = octave_marker | beat_group_indicator | syllable | space | unknown_lower
beat_group_indicator = "_" "_"+   // exactly two or more consecutive underscores (minimum length: 2)
syllable = letter+ (letter | digit | "'" | "-")*   // alphanumeric with apostrophes and hyphens
space = " "+
unknown_lower = !octave_marker !("_" "_") !space !syllable ANY+
```

### Octave Marker Semantics
```ebnf
// Unified octave marker - interpretation determined by spatial assignment rule
octave_marker = "." | ":"

// Spatial assignment rule determines meaning:
// assign_octave_marker(source: octave_marker, destination: note, distance: i8)
//   - distance < 0: upper line (octave increase)
//     - "." → +1 octave
//     - ":" → +2 octave
//   - distance > 0: lower line (octave decrease)
//     - "." → -1 octave
//     - ":" → -2 octave
//
// Rule-based interpretation eliminates need for separate upper/lower marker types
// Destination notation: [[ ]] indicates which line receives the modification
```

### Notation Systems

```ebnf
// Note: Melodic pitch systems support accidentals (#, ##, b, bb) appended to base notes when it makes sense

sargam_note = "S" | "R" | "G" | "M" | "P" | "D" | "N" |
              "s" | "r" | "g" | "m" | "p" | "d" | "n"

number_note = "1" | "2" | "3" | "4" | "5" | "6" | "7"

western_note = "A" | "B" | "C" | "D" | "E" | "F" | "G" |
               "a" | "b" | "c" | "d" | "e" | "f" | "g"

tabla_note = "dha" | "dhin" | "ta" | "ka" | "taka" | "trkt" | "ge" |
             "Dha" | "Dhin" | "Ta" | "Ka" | "Taka" | "Trkt" | "Ge" |
             "DHA" | "DHIN" | "TA" | "KA" | "TAKA" | "TRKT" | "GE"

hindi_note = "स" | "र" | "ग" | "म" | "प" | "ध" | "न"
```

### Accidentals: ASCII-Only Approach

**Design Decision**: For simplicity and to avoid Unicode complications, we use ASCII characters for accidentals:
- **Sharp**: `#` (ASCII 35) - not `♯` (Unicode U+266F)
- **Flat**: `b` (ASCII 98) - not `♭` (Unicode U+266D)
- **Double sharp**: `##`
- **Double flat**: `bb`

**Examples**: `1#`, `2b`, `S#`, `Cb`, `4##`, `Rbb`

**Rationale**: Unicode symbols can cause positioning and rendering issues across different systems. ASCII characters are universally supported and avoid character boundary complications in text processing.

**Future Consideration**: Unicode symbols (`♯`, `♭`) may be added as a display-only transformation in the JavaScript presentation layer, but the core parser and data model will remain ASCII-only for robustness.

### Barlines and Structure

```ebnf
barline = "|" | "||" | "|:" | ":|" | "|]"
line_number = digit+ "."
```

## Spatial Production Rules

### Beat Grouping
```ebnf
spatially-delimited-beat ::=
    [[ (pitch | dash) (space | beat-element)* ]]
    [ underscores                            ]
```

### Slur Grouping
```ebnf
spatially-delimited-slur ::=
    [ overscores                             ]
    [[ (pitch | dash) (space | beat-element)* ]]
```

### Octave Assignment
```ebnf
spatially-delimited-octave ::=
    [ octave_marker ]
    [[ content_pitch ]]
    [ octave_marker ]
```

### Other Spatial Relationships

The grammar has not yet been fully updated to formalize all spatial relationships, but the following spatial aspects exist in the current implementation and should be formalized using similar production rules:

- **Upper Octave Markers**: Dots and colons above notes to indicate higher octaves
  ```
  [ .  :     ]  (upper octave markers: +1, +2 octaves)
  [ S  R  G  ]  →  spatially-marked-octaves
  ```

- **Highest Octave Marker**: Colon above notes for maximum octave increase
  ```
  [    :     ]  (highest octave marker: +2 octaves)
  [ S  R  G  ]  →  spatially-marked-octaves
  ```

- **Lower Octave Marker**: Dot below notes to indicate lower octave
  ```
  [ S  R  G  ]  (notes)
  [ .        ]  (lower octave marker: -1 octave)
  ```

- **Lowest Octave Marker**: Colon below notes for maximum octave decrease
  ```
  [ S  R  G  ]  (notes)
  [    :     ]  (lowest octave marker: -2 octaves)
  ```

- **Ornaments**: Mordents, trills, and other decorations above notes
  ```
  [ ~  ~     ]  (ornament markers)
  [ S  R  G  ]  →  spatially-ornamented-notes
  ```

- **Syllables**: Lyrics or tabla bols aligned below notes
  ```
  [ S  R  G  ]  (notes)
  [ ta re ga ]  →  spatially-syllabled-notes
  ```

## Document Metadata

The document header contains optional metadata that appears before the musical content. This section handles titles, composer information, and musical directives.

### Header Structure

```ebnf
// Examples of valid headers:

// Title followed by directives
Amazing Grace
Author: John Newton
Key: G
Tempo: Andante

// Just directives (no title)
Author: John Newton
Tempo: 120
Key: G

// Just title (no directives)
Amazing Grace
```

### Supported Directives

| Directive | Purpose | Example |
|-----------|---------|---------|
| `Title` | Song title | `Title: Amazing Grace` |
| `Author` | Composer/arranger | `Author: John Newton` |
| `Tonic` | Tonal center | `Tonic: G` |
| `Key` | Key signature | `Key: G major` |
| `Tempo` | Tempo marking | `Tempo: 120` or `Tempo: Andante` |
| `Time` | Time signature | `Time: 4/4` |

### Parsing Rules

1. **First non-blank line determines format**:
   - Contains `|` → Parse as inline metadata
   - Contains `:` without `|` → Single directive
   - No `:` → Standalone title

2. **Subsequent lines** (until blank line):
   - Lines with `:` → Directives
   - Lines without `:` → Additional title text

3. **Blank line** → End of header, start of musical content

## Spatial Relationship Rules

### Octave Markers
- **Position determines direction**:
  - Pre-content annotation = raise octave (upper)
  - Post-content annotation = lower octave (lower)
- **Alignment**: Markers align spatially with content notes
- **Symbols**: `.` (single octave), `:` (double octave)

### Slur Indicators vs Beat Group Indicators
- **Slur Indicators**: `_____` in upper_line = musical phrasing (slur_indicator)
- **Beat Group Indicators**: `_____` in lower_line = rhythmic grouping (beat_group_indicator)
- **Same symbol, different semantic meaning based on spatial context**

### Lower Line Elements
- **Lower octave markers**: `.` (single octave down), `:` (double octave down)
- **Beat group indicators**: `__` (minimum 2) or more consecutive underscores for rhythmic grouping
- **Syllables**: Text elements for spatial alignment (lyrics, tabla bols)
- **Spaces**: For alignment with content above

### Syllable Assignment
- **In lower_line**: Syllables can appear for spatial alignment below notes
- **In lyrics_line**: Traditional lyric lines with syllable-to-note assignment
- **Format**: `he-llo world sing-ing` with hyphens and apostrophes supported

## Test Cases

### Stave-Only Document (no header)
```
Input: |1 2 3 4|
Expected: stave_only_document with single stave containing four beats
```

### Stave-Only Document (single note)
```
Input: 1
Expected: stave_only_document with single stave, single beat
```

### Header and Staves Document (title line)
```
Input:
        Amazing Grace        Bach
Author: John Newton

|1 2 3 4|
Expected: header_and_staves_document with title "Amazing Grace", author "Bach", directive "Author: John Newton", single stave
```

### Header and Staves Document (directive only)
```
Input:
Author: John Newton
Tempo: 120

|1 2 3 4|
Expected: header_and_staves_document with directives, single stave
```

### Rhythm Extensions
```
Input: |1-- 2- 3 4|
Expected: Beat 1: "1--", Beat 2: "2-", Beat 3: "3", Beat 4: "4"
```

### Spatial Octaves
```
Input:
    .   .
|1 2 3 4|
    .
Expected: Notes 1,3 raised octave, note 4 lowered octave
```

### Mordents and Ornaments
```
Input:
~   ~   ~
|1 2 3 4|
Expected: Mordents aligned above notes 1, 2, 3
Rendering: Use musical mordent symbol 𝆝 (&#x1D19D;) for GUI display
```

### Lower Line Elements
```
Input:
|1 2 3 4|
.   ___  dha
Expected: Lower octave marker on note 1, beat grouping on notes 3-4, syllable "dha" aligned with note 4
```

### Multiple Staves
```
Input:
|1 2 3 4|

|5 6 7 1|
Expected: Two separate staves
```

## Implementation Requirements

### Parser Architecture

#### Document-Level Parsing Strategy
1. **Try `header_and_staves_document` first**: Look for header patterns
2. **Fall back to `stave_only_document`**: If no header detected
3. **This resolves ambiguity**: Single notes like "1" are always stave content

#### Stave-Level Classification (existing classifier)
1. **Phase 1**: Parse structural elements within stave (content vs annotation)
2. **Phase 2**: Classify annotations as upper/lower based on position relative to content
3. **Error Handling**: Preserve line/column positions for semantic errors
4. **Token Indexing**: Every parsed token carries zero-based indexes
   - `index_in_line`: zero-based character offset from the start of its line
   - `index_in_doc`: zero-based character offset from the start of the document
   - Line and column remain 1-based for human readability

### Grammar Constraints
- `content_line` must only match lines with musical pitches/beats
- `annotation_line` uses negative lookahead `!content_line`
- Single notation system per document
- Whitespace significant for beat separation

### Semantic Classification
```rust
fn classify_annotation_line(line: &AnnotationLine, position: Position) -> LineType {
    match position {
        PreContent if contains_octave_markers(line) => UpperOctave,
        PostContent if contains_octave_markers(line) => LowerOctave,
        _ if contains_syllables(line) => Lyrics,
        _ => Mixed
    }
}
```

## Error Handling

### Parsing Errors
- Structural grammar violations
- Invalid character sequences
- Malformed barlines or measures

### Semantic Errors  
- Mixed notation systems in single document
- Misaligned spatial annotations
- Invalid octave marker placement

## Implementation Status

- ✅ **Basic grammar**: Document, stave, content line parsing
- ✅ **Notation systems**: Number, sargam, ABC, doremi support
- ✅ **Barlines**: All barline types implemented
- ✅ **Mordents**: "~" symbol implemented as upper line element
- 🚧 **Spatial annotations**: Architecture defined, implementation in progress
- 🚧 **Lyrics**: Grammar defined, assignment logic pending
- ⚠️ **Ornaments**: Syntax specified, sequences of pitches not yet implemented
- ⚠️ **Chords**: Grammar defined, implementation not yet started

## Acceptance Criteria

### Grammar Completeness
- [ ] All notation systems parse correctly
- [ ] Spatial relationships preserved in AST
- [ ] Error messages include precise locations
- [ ] Single-line and multi-stave documents supported

### Robustness
- [ ] Ambiguous input handled gracefully
- [ ] Mixed content classification works correctly
- [ ] Performance acceptable for large documents
- [ ] Memory usage bounded

### Compatibility
- [ ] Existing music-text documents parse unchanged
- [ ] New features backward compatible
- [ ] Clear migration path for deprecated syntax

---

*This specification drives the implementation of the music-text parser with formal grammar rules and comprehensive test requirements.*
