# Canvas Editor Grammar Specification

## Overview

This specification defines a simplified grammar for the canvas editor, supporting multiple pitch notation systems with inline rhythm representation. **Upper and lower lines are NOT supported** - all musical elements appear inline within content lines.

## Notation System Rules

**Single Notation System Per Document**: Each document can only contain one notation system, determined by the first content line encountered. All subsequent content lines must use the same notation system.

Supported notation systems:
- **Sargam**: S, R, G, M, P, D, N (with accidentals)
- **Number**: 1, 2, 3, 4, 5, 6, 7 (with accidentals)
- **Western**: A, B, C, D, E, F, G (with accidentals)
- **Bhatkhande**: स, र, ग, म, प, ध, न
- **Tabla**: dha, dhin, ta, ka, etc.

## Grammar Rules (EBNF)

### Document Structure

```ebnf
document = blank_lines* (single_content_line | header)* stave* blank_lines*

stave = content_line+

single_content_line = content_line

header = header_line+

header_line = title_line | directive_line | text_line

directive_line = directive whitespace* (newline | EOI)

directive = key ":" value

title_line = whitespace{3,} title whitespace{3,} author whitespace* (newline | EOI)

title = text_content

author = text_content

text_line = text_content (newline | EOI)

key = identifier

value = (ANY - newline)*

identifier = letter (letter | digit | "_")*

document_body = stave (blank_lines stave)*

blank_lines = newline (whitespace* newline)+
newline = "\n"
whitespace = " "
EOI = end_of_input
```

### Content Line Structure

```ebnf
content_line = content_element+ (newline | EOI)

content_element = beat | barline | whitespace | dash | unknown_token

beat = note_element+

note_element = note | dash

note = pitch octave_modifier*

pitch = pitch_char accidental?

pitch_char = sargam_pitch | number_pitch | western_pitch | bhatkhande_pitch | tabla_sound

octave_modifier = "." | "*"  // . = higher octave, * = lower octave (inline notation)

accidental = "#" | "b" | "'"

dash = "-"

barline = single_barline | double_barline | final_barline | repeat_start | repeat_end | repeat_both

single_barline = "|"
double_barline = "||"
final_barline = "|]"
repeat_start = "[:"
repeat_end = ":|"
repeat_both = "[:|"

whitespace = " "+

unknown_token = (!note !barline !whitespace !dash ANY)+
```

### Pitch Characters

```ebnf
// Sargam notation (Indian classical)
sargam_pitch = "S" | "R" | "G" | "M" | "P" | "D" | "N" | "s" | "r" | "g" | "m" | "p" | "d" | "n"

// Number notation (Indian folk/popular)
number_pitch = "1" | "2" | "3" | "4" | "5" | "6" | "7"

// Western notation
western_pitch = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "a" | "b" | "c" | "d" | "e" | "f" | "g"

// Bhatkhande notation (Devanagari)
bhatkhande_pitch = "स" | "र" | "ग" | "म" | "प" | "ध" | "न"

// Tabla sounds (percussion)
tabla_sound = "dha" | "dhin" | "ta" | "ka" | "na" | "tun" | "dheem" | "ge" | "ki" | "tak"
```

## Simplified Semantics

### Inline Octave Modifiers
- **Higher octave**: `S.` = S raised one octave, `S..` = S raised two octaves
- **Lower octave**: `S*` = S lowered one octave, `S**` = S lowered two octaves
- **No spatial alignment required** - modifiers appear directly after the note

### Beat Grouping
- **Spatial grouping**: Notes without spaces form beats: `SRG` = one beat with three notes
- **Beat separation**: Spaces separate beats: `SRG MPD` = two beats
- **No beat group indicators needed** - grouping is implicit from spacing

### Barlines
- **Inline barlines**: `SRG | MPD` inserts barline between beats
- **All barline types supported**: `|`, `||`, `|]`, `[:`, `:|`, `[:|`

### Content Examples

```
// Simple melody
SRGM PDNS

// With octave modifiers
S.RG M*PD

// With accidentals
S#RG MbPD

// With barlines
SRGM | PDNS |]

// Multi-line stave
SRGM PDNS
GMP* DN*S.

// Number notation
1234 5671

// Western notation
CDEF GABC

// Mixed with dashes (rhythm)
S-R- G-M-
```

## Implementation Notes

### Parser Simplifications
1. **No spatial alignment parsing** - no need to track upper/lower line positions
2. **No annotation line classification** - everything is inline within content lines
3. **Simplified beat detection** - based on whitespace, not spatial grouping indicators
4. **Direct octave attachment** - modifiers parse as part of note tokens

### Canvas Editor Benefits
1. **Easier typing** - no need for spatial alignment while typing
2. **Familiar inline syntax** - similar to existing text-based music formats
3. **Real-time parsing** - no complex multi-line state management
4. **Cursor positioning** - straightforward character-based cursor placement

### Limitations vs Full Grammar
1. **No complex spatial annotations** - slurs, ornaments, lyrics must be inline
2. **No separate lyric lines** - lyrics would need inline embedding
3. **Limited visual expressiveness** - trades spatial flexibility for simplicity

## Migration Path

For documents using the full spatial grammar, conversion rules:
- Upper octave markers `.` above notes → inline `.` after notes
- Lower octave markers `.` below notes → inline `*` after notes
- Beat group indicators `___` → remove, rely on spacing
- Slur indicators → convert to inline slur markers (future)

This simplified grammar enables a more accessible canvas editor while maintaining core musical expressiveness.