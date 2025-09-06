# Bhatkhande Sargam Notation Specification

## Overview

This specification defines the Bhatkhande sargam notation system, developed by Pandit Vishnu Narayan Bhatkhande (1860-1936). This system became the most widely adopted notation standard for Indian classical music in the 20th century.

## Historical Context

Pandit Vishnu Narayan Bhatkhande was a pioneering musicologist who:
- Introduced the first modern treatise on Indian classical music
- Reclassified scales into the currently used 10-scale system
- Created a systematic notation system that gained widespread acceptance
- Established a standardized way to document the rich oral tradition of Indian classical music

## Core Principles

### 1. Tonic-Centered System
Like the existing notation systems in this parser, Bhatkhande notation is **tonic-centered**:
- All notes are relative to the declared tonic (S)
- The tonic can be any absolute pitch (C, D, F#, etc.)
- Scale degrees maintain their relationship to the tonic regardless of transposition

### 2. Script Independence
- Originally written in Devanagari script
- Can be adapted to any script (Latin, regional scripts)
- Well-suited for internationalization and digitization

## Note System

### Basic Notes
The seven fundamental notes with their degree mappings:

| Note | Degree | Western Equiv | Description |
|------|---------|---------------|-------------|
| S    | N1 | Do/C | Tonic, fundamental note |
| R    | N2 | Re/D | Second degree |
| G    | N3 | Mi/E | Third degree |
| M    | N4 | Fa/F | Fourth degree |
| P    | N5 | Sol/G | Fifth degree |
| D    | N6 | La/A | Sixth degree |
| N    | N7 | Ti/B | Seventh degree |

## Octave Notation

### Traditional Markings
- **Upper octave**: Dot above the note → **:** in digital notation
- **Lower octave**: Dot below the note → **.** in digital notation  
- **Middle octave**: No marking (default)

### Digital Representation
Following the existing parser's octave system:

| Traditional | Digital | Degree | Description |
|-------------|---------|---------|-------------|
| Lower S     | S.      | N1 (octave -1) | Lower S |
| Middle S    | S       | N1 (octave 0)  | Middle S |
| Upper S     | S:      | N1 (octave +1) | Upper S |

**Examples**:
- `S. R G M P D N S S: R: G:` - Ascending across three octaves
- `N. S R G M P D N S` - Mixed octaves example

## Rhythmic Notation

### Rhythmic Structure Integration
The Bhatkhande system organizes notation around rhythmic cycles:

#### 16-beat cycle
Divided into four sections of 4 beats each:
```
| S R G M | P D N S | S N D P | M G R S |
  1 2 3 4   5 6 7 8   9 10 11 12  13 14 15 16
```

### Beat Subdivision
- **Whole beat**: Single note occupies one beat
- **Half beats**: Two notes per beat (`S R` in one beat)
- **Quarter beats**: Four notes per beat (`S R G M` in one beat)
- **Dashes (-)**: Extend previous note duration

### Rhythmic Examples
```
Basic 4-beat pattern:
S - R - | G - M - | P - D - | N - S -

Complex subdivision:
S R G M | P D N S | S - - - | R G M P
```

## Scale and Tonal Context

### Scale System Integration
Bhatkhande organized melodies into 10 parent scales:

1. **Scale 1**: S R G M P D N → Natural major scale
2. **Scale 2**: S R G M P D N → Mixolydian mode  
3. **Scale 3**: S R G M P D N → Natural minor scale
4. **Scale 4**: S R G M P D N → Dorian mode
5. **Scale 5**: S R G M P D N → All seven notes
6. **Scale 6**: S R G M P D N → Lydian mode
7. **Scale 7**: S R G M P D N → Unique combination
8. **Scale 8**: S R G M P D N → Distinctive pattern
9. **Scale 9**: S R G M P D N → Complex arrangement
10. **Scale 10**: S R G M P D N → Traditional pattern

### Tonic Declaration
Like other systems in this parser:
```
key: C → S = C (C major base)
key: D → S = D (D as tonic)
key: F# → S = F# (F# as tonic)
```

## Integration with Existing Parser Architecture

### Notation Enum Extension
Add to `src/models/pitch.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Western,
    Number,
    Sargam,
    Tabla,
    Bhatkhande,  // New notation system
}
```

### Degree Mappings
Reuse existing `Degree` enum with Bhatkhande-specific lookup:

```rust
// In lookup_pitch() function
Notation::Bhatkhande => match symbol {
    // Basic notes
    "S" => Some(Degree::N1),
    "R" => Some(Degree::N2),
    "G" => Some(Degree::N3),
    "M" => Some(Degree::N4),
    "P" => Some(Degree::N5),
    "D" => Some(Degree::N6),
    "N" => Some(Degree::N7),
    
    // Extended accidentals (if needed)
    "S#" => Some(Degree::N1s),
    "R#" => Some(Degree::N2s),
    // ... etc
}
```

### Octave Processing
Extend existing octave marker functions in `src/models/pitch.rs`:

```rust
pub fn parse_bhatkhande_octave(symbol: &str) -> i8 {
    let mut octave = 0i8;
    
    for ch in symbol.chars() {
        match ch {
            '.' => octave -= 1,   // Lower octave (ṣa)
            ':' => octave += 1,   // Upper octave (sa̅)
            _ => {}
        }
    }
    
    octave
}
```

## Syntax Examples

### Basic Melodic Phrases
```
// Ascending scale
S R G M P D N S:

// Descending scale  
S: N D P M G R S

// Simple ascending scale
S R G M P D N S
```

### Rhythmic Patterns
```
// 16-beat composition
beat: 16
S - G - | M - P - | S - G - | M P G R |
S - - - | R - G - | M - P - | D N S - |
```

### Complex Musical Phrases
```
// Sample composition
key: C
beat: 16

N. R G M | G R S - | N. R G M | P - G R |
M P D N | S: - N D | P M G R | S - - - |
```

### Rhythm variations
```
// Double speed
S R G M P D N S | R G M P D N S R |

// Half speed
S - - - | R - - - | G - - - | M - - - |
```

## Special Symbols and Ornaments

### Ornaments
- **Glide**: `S~R` or `S-R` with curved line
- **Quick oscillation**: `S^` or `(S R S)`
- **Grace note**: `(G)M` - small note before main note
- **Oscillation**: `S~~` or `S±`

### Breath Marks and Phrasing
- **Breath mark**: `'` (apostrophe) - short pause
- **Long pause**: `||` - section break
- **Phrase boundaries**: `()` - grouping notes

### Dynamic and Expression Marks
- **Crescendo**: `<`
- **Decrescendo**: `>`
- **Emphasis**: `*S*` - stressed note
- **Sustain**: `S---` - hold the note

## Compatibility with Existing Systems

### Sargam System Overlap
The Bhatkhande system shares significant overlap with the existing Sargam notation:
- Both use S, R, G, M, P, D, N
- Both use similar note systems
- Both use similar octave markers

### Key Differences from Sargam
1. **Theoretical Framework**: Bhatkhande includes the 10-scale system
2. **Rhythmic Integration**: Stronger emphasis on rhythmic structure
3. **Standardization**: More systematized approach to notation
4. **Historical Context**: Specific to early 20th century reform movement

### Number System Compatibility  
Both systems map to the same internal `Degree` representation:
```
Bhatkhande: S  R  G  M  P  D   N
Number:     1  2  3  4  5  6   7
Degree:     N1 N2 N3 N4 N5 N6  N7
```

## Implementation Notes

### Parser Integration Points
1. **Tokenizer Extension** (`src/parser/tokenizer.rs`):
   - Add Bhatkhande-specific token patterns
   - Handle long-form names (Sa, Re, Ga) vs short-form (S, R, G)
   - Process octave markers and ornaments

2. **Notation Detection** (`src/parser/notation_detector.rs`):
   - Add Bhatkhande pattern recognition
   - Distinguish from existing Sargam system
   - Handle mixed notation detection

3. **Pitch Lookup Extension** (`src/models/pitch.rs`):
   - Extend `lookup_pitch()` function
   - Add Bhatkhande-specific mappings
   - Handle alternative spellings

### Test Cases
```rust
#[test]
fn test_bhatkhande_basic_swaras() {
    assert_eq!(lookup_pitch("Sa", Notation::Bhatkhande), Some(Degree::N1));
    assert_eq!(lookup_pitch("re", Notation::Bhatkhande), Some(Degree::N2b));
    assert_eq!(lookup_pitch("M#", Notation::Bhatkhande), Some(Degree::N4s));
}

#[test]
fn test_bhatkhande_octaves() {
    // Test octave processing
    assert_eq!(parse_bhatkhande_octave("Sa:"), 1);  // Upper Sa
    assert_eq!(parse_bhatkhande_octave("Sa."), -1); // Lower Sa
    assert_eq!(parse_bhatkhande_octave("Sa"), 0);   // Middle Sa
}
```

## Conclusion

The Bhatkhande notation system provides a comprehensive, historically significant approach to notating Indian classical music. Its systematic organization of ragas, emphasis on rhythmic structure, and script-independent design make it well-suited for digital implementation and international use.

By integrating Bhatkhande notation into the existing parser architecture, we can:
- Preserve an important musical heritage
- Provide musicians with a standardized notation system
- Enable cross-system musical analysis and conversion
- Support the rich theoretical framework of Hindustani classical music

The system's tonic-centered approach aligns perfectly with the existing parser's architecture, making integration straightforward while maintaining the unique characteristics that made Bhatkhande's system historically influential.