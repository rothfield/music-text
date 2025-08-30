# music-text Input Language Specification

*Text-based musical notation input language*

## Overview

music-text is a plain text format for writing musical notation using letters, numbers, and symbols. It supports multiple pitch notation systems with proportional rhythm representation.

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

**Simple beats** - no spaces allowed inside:
```
123   SRG   ABC
```

**Delimited beats** - spaces allowed inside:
```
<1 2 3>   <S R G>   <A B C>
```

### Slur Notation

**Underscores** `_` create slurred groups:
```
_
1 2 3     // Slur over notes 1, 2, 3
```

### Measures and Barlines

**Barlines** `|` separate measures:
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

### Metadata Section (Optional)
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

### With metadata and barlines
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

### Delimited beats
```
<1 2> <3- 4> <5>
```

## Grammar Summary

```
document     ::= metadata* music_lines
metadata     ::= key ':' value newline  
music_lines  ::= (octave_line | pitch_line)*
octave_line  ::= octave_markers
pitch_line   ::= measures
measures     ::= beat+ ('|' beat+)*
beat         ::= simple_beat | '<' complex_beat '>'
simple_beat  ::= (pitch | dash | slur)+
complex_beat ::= (pitch | dash | slur | space)+
pitch        ::= pitch_char accidental?
octave_markers ::= ('.' | '*' | ':')*
```

## Rules

1. **Single notation system** per document
2. **Spaces separate beats** in pitch lines  
3. **Vertical alignment** for octave markers
4. **Proportional rhythm** via dash counting
5. **Slur scope** covers following pitch sequence

This specification covers the core input syntax for music-text musical notation.