# Music Text Grammar Specification

## Overview

This specification defines the formal grammar for the music-text notation language, supporting multiple pitch notation systems with spatial rhythm representation and hierarchical document structure.

## Grammar Rules (EBNF)

### Document Structure

```ebnf
document = blank_lines* (stave (blank_lines stave)*)? blank_lines?

stave = upper_line* content_line (lower_line | lyrics_line)* (blank_lines | (whitespace* newline)* EOI)

blank_lines = newline (whitespace* newline)+
newline = "\n"
whitespace = " "
```

### Content Lines

```ebnf
content_line = line_number? barline? beat+ barline? newline

beat = spatially-delimited-beat |
      (pitch | dash) beat-element*
beat-element = pitch | dash | breath-mark

pitch = note_in_system

note_in_system = sargam_note | number_note | western_note | tabla_note | hindi_note
dash = "-"
breath-mark = ","
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
upper_line_element = upper_octave_marker | slur_indicator | ornament | chord | mordent | tala | space | unknown_upper

upper_octave_marker = "." | "*" | ":"
slur_indicator = "_" "_"+   // exactly two or more consecutive underscores for slur marking (minimum length: 2)
ornament = "<" pitch+ ">" | pitch+
chord = "[" chord_symbol "]"
mordent = "~"
tala = "+" | "0" | digit
space = " "+
unknown_upper = !upper_octave_marker !("_" "_") !space !ornament !chord !mordent !tala ANY+
```

### Lower Line Elements

```ebnf
lower_line_element = lower_octave_marker | beat_group_indicator | syllable | space | unknown_lower

lower_octave_marker = "." | ":"
beat_group_indicator = "_" "_"+   // exactly two or more consecutive underscores (minimum length: 2)
syllable = letter+ (letter | digit | "'" | "-")*   // alphanumeric with apostrophes and hyphens
space = " "+
unknown_lower = !lower_octave_marker !("_" "_") !space !syllable ANY+
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

### Barlines and Structure

```ebnf
barline = "|" | "||" | "|:" | ":|" | "|]"
line_number = digit+ "."
```

## Spatial Production Rules

### Beat Grouping
```ebnf
spatially-delimited-beat ::=
    [ (pitch | dash) (space | beat-element)* ]
    [ underscores                            ]
```

### Slur Grouping
```ebnf
spatially-delimited-slur ::=
    [ overscores                             ]
    [ (pitch | dash) (space | beat-element)* ]
```

### Other Spatial Relationships

The grammar has not yet been fully updated to formalize all spatial relationships, but the following spatial aspects exist in the current implementation and should be formalized using similar production rules:

- **Octave Markers**: Dots or colons above/below notes to indicate octave changes
  ```
  [ .  :     ]  (upper octave markers)
  [ S  R  G  ]  →  spatially-marked-octaves
  [    .     ]  (lower octave markers)
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

## Spatial Relationship Rules

### Octave Markers
- **Position determines direction**: 
  - Pre-content annotation = raise octave (upper)
  - Post-content annotation = lower octave (lower)
- **Alignment**: Markers align spatially with content notes
- **Symbols**: `.` (single octave), `:` (double octave), `*` (alternative)

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

### Document with Title/Author
```
Input: 
Title: Amazing Grace | Author: John Newton
Tonic: G

|1 2 3 4|
Expected: Title "Amazing Grace", Author "John Newton", directives, single stave
```

### Document with Separate Title Line
```
Input: 
Amazing Grace
Author: John Newton
Tonic: G

|1 2 3 4|
Expected: Title "Amazing Grace", directives, single stave
```

### Basic Notation
```
Input: |1 2 3 4|
Expected: Four beats separated by spaces
```

### Rhythm Extensions
```
Input: |1-- 2- 3 4|
Expected: Beat 1: "1--", Beat 2: "2-", Beat 3: "3", Beat 4: "4"
```

### Spatial Octaves
```
Input: 
    •   •
|1 2 3 4|
    •
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
1. **Phase 1**: Parse structural elements (content vs annotation)
2. **Phase 2**: Classify annotations based on position and content
3. **Error Handling**: Preserve line/column positions for semantic errors

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