# Music Text Grammar Specification

## Overview

This specification defines the formal grammar for the music-text notation language, supporting multiple pitch notation systems with spatial rhythm representation and hierarchical document structure.

## Grammar Rules (EBNF)

### Document Structure

```ebnf
document = metadata_line? directives_section? stave+

metadata_line = title_author_line | title_line
title_author_line = "Title:" title_value "|" "Author:" author_value newline
title_value = (letter | digit | punctuation | space)+
author_value = (letter | digit | punctuation | space)+
title_line = text_line newline
text_line = (letter | digit | punctuation | space)+

directives_section = directive+
directive = identifier ":" value newline

stave = annotation_line* content_line annotation_line*
```

### Content Lines

```ebnf
content_line = line_number? barline? measure (barline measure)* barline? newline

measure = beat+
beat = simple_beat | delimited_beat

simple_beat = (pitch | dash | space)+
delimited_beat = overline_marker newline (pitch | dash | space)+ newline

pitch = pitch_char accidental?
pitch_char = sargam_note | number_note | abc_note | doremi_note | hindi_note
accidental = "#" | "b"
dash = "-"
space = " "
```

### Annotation Lines

```ebnf
annotation_line = !content_line annotation_item+ newline

annotation_item = octave_marker | slur | ornament | chord | tala | syllable | whitespace

octave_marker = "." | "*" | ":"
slur = "_"+
ornament = "<" pitch+ ">" | pitch+
chord = "[" chord_symbol "]"
tala = "+" | "0" | digit
syllable = letter+ ("-" | whitespace)*
whitespace = " "+
```

### Notation Systems

```ebnf
sargam_note = "S" | "R" | "G" | "M" | "P" | "D" | "N" | 
              "s" | "r" | "g" | "m" | "p" | "d" | "n"

number_note = "1" | "2" | "3" | "4" | "5" | "6" | "7"

abc_note = "A" | "B" | "C" | "D" | "E" | "F" | "G" |
           "a" | "b" | "c" | "d" | "e" | "f" | "g"

doremi_note = "D" | "R" | "M" | "F" | "S" | "L" | "T" |
              "d" | "r" | "m" | "f" | "s" | "l" | "t"

hindi_note = "स" | "र" | "ग" | "म" | "प" | "ध" | "न"
```

### Barlines and Structure

```ebnf
barline = "|" | "||" | "|:" | ":|" | "|]"
line_number = digit+ "."
```

## Spatial Relationship Rules

### Octave Markers
- **Position determines direction**: 
  - Pre-content annotation = raise octave (upper)
  - Post-content annotation = lower octave (lower)
- **Alignment**: Markers align spatially with content notes
- **Symbols**: `.` (single octave), `:` (double octave), `*` (alternative)

### Slurs vs Beat Groups
- **Slurs**: `_____` in pre-content = musical phrasing
- **Beat Groups**: `_____` in post-content = rhythmic grouping  
- **Same symbol, different semantic meaning based on spatial context**

### Lyrics Assignment
- **Syllables**: Text broken into syllables with hyphens
- **Alignment**: Spatial alignment with notes or auto-assignment
- **Format**: `he-llo world sing-ing`

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
Expected: Single measure with four quarter notes
```

### Rhythm Extensions
```
Input: |1-- 2- 3 4|  
Expected: 1 (dotted half), 2 (quarter), 3 (eighth), 4 (eighth)
```

### Spatial Octaves
```
Input: 
    •   •
|1 2 3 4|
    •
Expected: Notes 1,3 raised octave, note 4 lowered octave
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
- 🚧 **Spatial annotations**: Architecture defined, implementation in progress
- 🚧 **Lyrics**: Grammar defined, assignment logic pending
- 🚧 **Ornaments**: Syntax specified, rendering pending

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