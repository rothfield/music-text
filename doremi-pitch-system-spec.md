# Do-Re-Mi Pitch System Specification

## Overview
The Do-Re-Mi pitch system is a traditional Western musical notation system using syllables to represent scale degrees. This specification outlines the implementation of the Do-Re-Mi pitch system in the Music-Text notation parser.

## Syntax

### Basic Syllables
- `Do` / `do` - Scale degree 1 (tonic)
- `Re` / `re` - Scale degree 2 (supertonic)
- `Mi` / `mi` - Scale degree 3 (mediant)
- `Fa` / `fa` - Scale degree 4 (subdominant)
- `Sol` / `sol` - Scale degree 5 (dominant)
- `La` / `la` - Scale degree 6 (submediant)
- `Ti` / `ti` - Scale degree 7 (leading tone)

### Alternative Spellings
Support for common alternative spellings:
- `So` - Alternative for `Sol`
- `Si` - Alternative for `Ti` (European convention)

### Case Insensitive
The system should accept both uppercase and lowercase variations:
- `DO`, `do`, `Do`
- `RE`, `re`, `Re`
- etc.

## Notation Examples

### Basic Scale
```
| do re mi fa | sol la ti do |
```

### With Octave Markers
Higher octaves (dots above):
```
  . . . .
| do re mi fa |
```

Lower octaves (dots below):
```
| do re mi fa |
  . . . .
```

### Mixed Case Usage
```
| Do Re Mi Fa | Sol La Ti Do |
```

### Polyphonic Example
```
Row row row your boat
###
| do do do re | mi - - - |

| mi mi re re | do - - - |
###
```

## Implementation Notes

### Parser Integration
- Add Do-Re-Mi syllables to the pitch system enum
- Implement parsing logic in the content line parser
- Map syllables to appropriate pitch codes for rendering

### Pitch Mapping
Map Do-Re-Mi syllables to chromatic pitches:
- `do` → C
- `re` → D
- `mi` → E
- `fa` → F
- `sol` → G
- `la` → A
- `ti` → B

### Rendering Support
- LilyPond renderer: Convert to Western note names
- VexFlow renderer: Convert to appropriate pitch representations
- ASCII renderer: Display syllables as entered

## Compatibility
The Do-Re-Mi system should work alongside existing pitch systems:
- Sargam: `| S R G M |`
- Numbers: `| 1 2 3 4 |`
- Western: `| C D E F |`
- Do-Re-Mi: `| do re mi fa |`

## Testing
Include test cases for:
- Basic syllable recognition
- Case variations
- Octave markers
- Polyphonic notation
- Mixed pitch systems (error handling)
- Alternative spellings (So, Si)

## Future Enhancements
- Support for chromatic variants (di, ra, me, etc.)
- Movable Do vs Fixed Do system selection
- International syllable variations