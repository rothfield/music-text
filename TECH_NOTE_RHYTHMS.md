# Tech Note: Beat Subdivision and LilyPond Rhythm Conversion

## Overview

This document describes the rhythm system used in the Indian classical notation parser and how it converts to LilyPond durations.

## Beat Structure

### Beat Boundaries
- **Beats are delimited by spaces and barlines**
- Each beat always equals exactly **1/4 note duration** in LilyPond
- Different beats can have different numbers of internal subdivisions

### Subdivision System
- Each **pitch** (S, R, G, m, P, d, n, etc.) = **1 subdivision**
- Each **dash** (-) = **1 subdivision** 
- Total subdivisions in a beat determine the subdivision value

### Duration Calculation
```
subdivision_value = (1/4) ÷ (total_subdivisions_in_beat)
note_duration = (note_subdivisions) × subdivision_value
```

## Dash Behavior

### Extension Rule
**Dashes extend the immediately preceding pitch, even across beat boundaries**

### Rest Rule  
**Only create rests if there is no preceding pitch to extend**

## Examples

### Single Beat Examples
```
S---     = 4 subdivisions = S gets 4/4 of beat = 1/4 note = c'4
S--      = 3 subdivisions = S gets 3/3 of beat = 1/4 note = c'4  
S- R     = 3 subdivisions = S gets 2/3, R gets 1/3 = fraction_to_lilypond(2,12) + fraction_to_lilypond(1,12)
-S       = 2 subdivisions = rest gets 1/2, S gets 1/2 = r8 c'8
```

### Multi-Beat Examples
```
S- R     = 2 beats
         Beat 1: S- (S extended) = c'4
         Beat 2: R = d'4

S- -R    = 2 beats  
         Beat 1: S- (S extended) = c'4~
         Beat 2: -R (dash continues S, then R) = c'8 d'8
         LilyPond: c'4~ c'8 d'8

-S --R   = 2 beats
         Beat 1: -S (rest then S) = r8 c'8  
         Beat 2: --R (rest then R) = r8. d'16 (or equivalent)
```

### Complex Example
```
S-- R- | G--- -P
= 4 beats (space and barline separators)

Beat 1: S-- (3 subdivisions)
  - S gets 3 subdivisions = 3/3 of beat = c'4

Beat 2: R- (2 subdivisions)  
  - R gets 2 subdivisions = 2/2 of beat = d'4

Beat 3: G--- (4 subdivisions)
  - G gets 4 subdivisions = 4/4 of beat = e'4

Beat 4: -P (2 subdivisions)
  - Dash extends G from previous beat = e'4~ e'8
  - P gets 1 subdivision = g'8
  - LilyPond for beats 3-4: e'4~ e'8 g'8
```

## Notation Approach

### Unified Fraction-Based System
All beat subdivisions are handled using **fraction calculations** with the existing `fraction_to_lilypond()` function. No special cases needed for different subdivision counts.

### Core Principle
- Each beat = **1/4 note** (regardless of subdivision count)
- Each note gets: `(note_subdivisions / total_subdivisions) × (1/4 note)`
- Convert fraction to LilyPond using existing lookup table and tied note logic

### Examples Across Different Subdivision Counts
```
2 subdivisions:
S R     = S gets 1/8, R gets 1/8 = c'8 d'8

3 subdivisions:
S R G   = S gets 1/12, R gets 1/12, G gets 1/12 = fraction_to_lilypond(1,12) each
S- R    = S gets 2/12, R gets 1/12 = fraction_to_lilypond(2,12) + fraction_to_lilypond(1,12)
S--     = S gets 3/12 = fraction_to_lilypond(3,12) = c'4

4 subdivisions:
S R G m = S gets 1/16, R gets 1/16, etc. = c'16 d'16 e'16 f'16

5 subdivisions:  
S---r   = S gets 4/20, r gets 1/20 = fraction_to_lilypond(4,20) + fraction_to_lilypond(1,20)
```

## LilyPond Conversion Algorithm

1. **Parse beats**: Split input by spaces and barlines
2. **Count subdivisions**: Sum all pitches and dashes in each beat  
3. **Group elements**: 
   - Pitches with their following dashes
   - Leading dashes (no preceding pitch) become rests
4. **Calculate fractions**: For each element in beat:
   - `element_fraction = element_subdivisions / total_subdivisions`
   - `lilypond_fraction = element_fraction × (1/4)`
   - `duration = fraction_to_lilypond(numerator, denominator)`
5. **Handle ties**: When dashes extend across beat boundaries, use LilyPond ties (`~`)
6. **Combine beats**: Join with barlines and handle cross-beat extensions

## Fraction Calculation Details

### Beat Duration Calculation
```
beat_duration = 1/4 note (always, regardless of subdivisions)
element_duration = (element_subdivisions / total_subdivisions) × (1/4)

Examples:
3 subdivisions: each subdivision = 1/3 × 1/4 = 1/12 note
5 subdivisions: each subdivision = 1/5 × 1/4 = 1/20 note  
7 subdivisions: each subdivision = 1/7 × 1/4 = 1/28 note
```

### Fraction Simplification
The `fraction_to_lilypond()` function handles complex fractions by:
- Using lookup table for common fractions (1/4 → "4", 3/8 → "4.", etc.)
- Decomposing complex fractions into tied notes (7/12 → "4 ~ 16")
- Falling back to smallest unit ties for very complex ratios

## Fraction to LilyPond Duration

For standard (power-of-2) beats, complex fractions are handled using the `fraction_to_lilypond()` function which:
- Maps common fractions directly (1/4 → "4", 1/8 → "8", 3/8 → "4.", etc.)
- Decomposes complex fractions into tied notes
- Example: 7/12 → "4 ~ 16" (quarter tied to sixteenth)

## Key Insights

1. **Consistent beat duration**: Every beat = 1/4 note, regardless of subdivisions
2. **Flexible subdivision**: Beats can have 2, 3, 4, 5+ subdivisions  
3. **Natural phrasing**: Dashes create sustained notes across beat boundaries
4. **Minimal rests**: Rests only occur when no preceding pitch exists
5. **Cross-boundary extension**: Notes can sustain across multiple beats via dashes

This system allows for complex rhythmic patterns while maintaining a consistent quarter-note pulse structure that maps cleanly to standard Western notation via LilyPond.