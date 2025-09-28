# Music Text Language Specification


## Overview


## Document Structure

### Complete Hierarchical Organization

music-text follows 

```
Document
├── DirectivesSection (optional - document-level Directives)
│   ├── Metadata Line (Title: Amazing Grace | Author: John Newton OR standalone title)
│   ├── Key: C / D / Bb (🚧 planned)
│   ├── Tonic: C / D / Bb (movable-do system - 🚧 planned)
│   ├── Tempo: 120 (🚧 planned)
│   └── TimeSignature: 4/4 (🚧 planned)
│
    ├── ContentLine (main musical content: |1 2 3 4|)
    │   └── ContentElement
    │       ├── Note (1, 2, 3... with accidentals 1# 2b)
    │       ├── Barline (|, ||, |:, :|, |])
    │       ├── Space (beat separator)
    │       ├── Dash (rhythm extender -)
    │       └── SlurMarker (() (❌ deprecated - use spatial slurs)
    │
    ├── UpperLine (spatial annotations above content, multiple possible)
    │       ├── UpperOctaveMarker (• typed as ., :: typed as : for highest octave)
    │       ├── Slur (_______ underscores for phrasing)
    │       ├── Ornament (123, <456> grace notes/melismas - 🚧 planned)
    │       ├── Chord ([Am] chord symbols - 🚧 planned)
    │       ├── Tala (+, 0, 2 tala markers - 🚧 planned)
    │       ├── Mordent (~ trill symbol - 🚧 planned)
    │       └── Ending (1.___ 2.___ repeat endings - 🚧 planned)
    │
    ├─spatial annotations below content, multiple possible)
    │   └── LowerElement
    │       ├── LowerOctaveMarker (• typed as ., :: typed as : for lowest octave)
    │       ├── BeatGroup  -- indicated by lower loops in mss .
    │       └── FlatMarker (_ flat marker, Bhatkande notation only - 🚧 planned)
    │
    └── LyricsLine (syllables, multiple possible - 🚧 planned)
        └── Syllable (he-llo, world, etc. - must align with notes for assignment)
```

### Document Processing

The hierarchical structure supports systematic processing:

```
   The mss typically have a header
   title
   text describing background raga additional info

    ├── Content Line (main musical content)
    ├── Upper Line (annotations above)
    ├── Lower Line (annotations below)
    └── Lyrics Line (syllables)
```

### Format Overview

music-text documents contain:
- **DirectivesSection**: Optional Directives (title, tonic, tempo)
- **Staves**: Musical content with spatial annotations

### Spatial Relationship Rules

**Octave Markers**:
- **Position determines direction**: UpperLine = raise octave, LowerLine = lower octave
- **Alignment**: Markers align spatially with content notes
- **Visual vs Text**: Display as bullets (•) but typed as dots (.), colon (:) for highest/lowest octaves

**Slurs vs Beat Groups**:
- **Slurs**: upper loos in UpperLine = musical phrasing (legato)
- **Beat Groups**: lowerloops in LowerLine = rhythmic grouping


**Ornaments** (🚧 planned):
- **Non-metrical**: No dashes, pure pitch sequences
- **Types**: Grace notes, melismas, ornamental passages
- **Format**: `123` or `<456>` consecutive number pitches
- **Musical**: Supports grace notes, melismatic decoration, and ornamental runs
- in mss have dots.

**Repeat Phrases** (🚧 planned):
  -- from mss
- **Simple repeats**: `(123)3x` - repeat pattern 3 times
- **Tihai notation**: `(12|3)3x` - repeat phrase with internal structure¹
- **Format**: Parentheses contain repeated material, followed by count and 'x'
- **Traditional barlines**: `|:` and `:|` (✅ implemented in doremi-script)
- **Measure repeat**: `%` (✅ implemented in doremi-script)

---

¹ **Tihai**: A rhythmic device in Indian classical music where a phrase is repeated three times, often with internal divisions marked by barlines within the repeat structure.

## Basic Elements

### Pitch Notation Systems

**Sargam (Indian Classical)**
- Basic notes: `S R G M P D N` (uppercase)
- Flat variants: `r g m d n` (lowercase)
- Accidentals: `S# Rb G# P#` etc.

**Number Notation**
- Scale degrees: `1 2 3 4 5 6 7`
- Accidentals: `1# 2b 3# 4b` etc.

**ABC (Western)**
- Letter names: `C D E F G A B`
- Accidentals: `C# Db E# Fb` etc.

**DoReMi/Solfège**
- Lowercase: `d r m f s l t`
- Uppercase: `D R M F S L T`
- Accidentals: `d# rb m# fb` etc.

**Hindi/Devanagari**
- Unicode: `स र ग म प ध न`
- Sharp Ma: `म'` (with apostrophe)

## Examples

### Simple Number Notation
```
|1 2 3 4| 5 6 7 1|
```

### With Rhythm Extensions
```
|1-2 3-- 4| --5 6-7 1|
```

### With Octave Markers
```
    •   •          ← Upper octave markers
|1-2 3-- 4| --5 6-7 1|
        •          ← Lower octave marker
```

### With Slurs
```
____   ____        ← Slurs (upper line)
|1-2 3 4| 5 6-7 1|
```

### With Metadata (Combined Format)
```
Title: Amazing Grace | Author: John Newton
Tonic: G

|1- 3 5 1|
|2- 1 7 5| 1 - - -|
```

### With Metadata (Legacy Format)
```
Amazing Grace
Author: John Newton
Tonic: G

|1- 3 5 1|
|2- 1 7 5| 1 - - -|
```

### Complete Song Example
```
Simple Song
Tonic: C

•   ::
|1- 2 3 4|
•     ::
he- llo world to- day

____
|5- 4 3 2|
•
sing- ing hap- py songs
```

### Multi-Notation Systems
```
Number:  |1 2 3 4| 5 6 7 1|
Sargam:  |S R G M| P D N S|
Western: |C D E F| G A B C|
```

### Planned Features
```
  123              ← Ornament (grace notes)
|1  2  3  4|       ← Main melody
  ___              ← Beat grouping

(123)3x            ← Repeat phrase (tihai)
|: 1 2 3 :|        ← Traditional repeat brackets
```

## Complete Document Example

```
Document                           | Specification Element
-----------------------------------|------------------------------------
Title: Amazing Grace | Author: John Newton  | Metadata Line (combined format)
Tonic: G                          | Directive
Tempo: 60                         | Directive
                                  |
•   •  •   •                      | UpperLine → UpperOctaveMarker
|1- 3 5 1|                        | ContentLine → Note, Dash, Barline
    •  •                          | LowerLine → LowerOctaveMarker
A- ma- zing Grace                 | LyricsLine → Syllable (with hyphens)
                                  |
____     ____                     | UpperLine → Slur (underscores)
|2- 1 7 5| 1 - - -|               | ContentLine → Note, Dash, Space, Barline
     ___                          | LowerLine → BeatGroup (underscores)
How sweet the sound               | LyricsLine → Syllable (spatial alignment)
                                  |
(567)2x                           | RepeatPhrase (🚧 planned)
|: 1 3 5 :| % |                   | Traditional repeats + measure repeat
```

### Rhythm Notation

**Dashes** `-` represent rhythmic placeholders:
```
1--2-3    // 1 gets 3 units, 2 gets 2 units, 3 gets 1 unit
```

**Spaces** ` ` separate beats:
```
1-2 3-4   // Two beats: (1-2) and (3-4)
```

### Beat Grouping

**Simple beats** - no spaces allowed inside (most common):
```
123   SRG   ABC
```

**Delimited beats** - spaces allowed inside, using overlines:
```
_____
1 2 3     // Overline above creates delimited beat
```

**Note**: The `<1 2 3>` bracket syntax is deprecated. Use overlines instead.

### Slur Notation

**Underscores** `_` create slurred groups:
```
_
1 2 3     // Slur over notes 1, 2, 3
```

### Segments and Barlines

**Barlines** `|` separate segments:
```
1 2 3 4 | 5 6 7 1 |
```

### Octave Indicators

**Dots** `.` and **asterisks** `*` indicate octave changes:
```
  . .
1 2 3 4    // Dots above raise octave
. .
```

**Colons** `:` for double octave changes:
```
  : :
1 2 3 4    // Raise two octaves
```

## Document Structure

### Directives Section (Optional)
```
Key: D major
Time: 4/4
Author: Composer Name
Title: Song Title
```

### Musical Content
Multiple lines of musical notation with optional octave indicator lines.

## Complete Examples

### Simple melody
```
1 2 3 4 5 6 7 1
```

### With rhythm
```
1-- 2- 3 4-
```

### With slurs
```
_   _
1-2 3-4
```

### With Directives and segments
```
Key: G
Time: 3/4

1 2 3 | 4 5 6 | 7 1 2 |
```

### Multi-line with octaves
```
    .
1 2 3 4
5 6 7 1
.
```

### Delimited beats (deprecated syntax)
```
<1 2> <3- 4> <5>    // DEPRECATED - use overlines instead
```

## Grammar Summary

```
document     ::= Directives* music_lines
Directives     ::= key ':' value newline
music_lines  ::= (octave_line | pitch_line)*
octave_line  ::= octave_markers
pitch_line   ::= segments
segments     ::= beat+ ('|' beat+)*
beat         ::= simple_beat    // Most beats are simple (undelimited)
simple_beat  ::= (pitch | dash | slur)+
pitch        ::= pitch_char accidental?
octave_markers ::= ('.' | '*' | ':')*

// Delimited beats use overlines (beat_grouping in lower_annotation_line)
// The '<' complex_beat '>' syntax is deprecated
```

## Rules

1. **Single notation system** per document
2. **Spaces separate beats** in pitch lines
3. **Vertical alignment** for octave markers
4. **Proportional rhythm** via dash counting
5. **Slur scope** covers following pitch sequence

This specification covers the core input syntax for music-text musical notation.
