# VexFlow Beaming Specification

## Overview
Beaming connects multiple notes with horizontal beams instead of individual flags. This is standard music notation practice for improving readability.

## Beamable Notes
Notes that should be beamed:
- Eighth notes (8)
- Sixteenth notes (16) 
- Thirty-second notes (32)
- Sixty-fourth notes (64)

Notes that should NOT be beamed:
- Whole notes (w)
- Half notes (h)
- Quarter notes (q)
- Dotted notes of any duration

## Beaming Rules

### 1. Beat-Based Beaming
- Notes should be beamed together WITHIN a beat
- Beam groups should NOT cross beat boundaries
- Each beat should have its own beam group if it contains beamable notes

### 2. Examples

#### Pattern: "1111" (four sixteenth notes in one beat)
- Input: `| 1111 |`
- Beat structure: 1 beat with 4 subdivisions
- Each note duration: 1/16
- Expected: All 4 sixteenth notes beamed together as one group
- VexFlow: `new VF.Beam([note0, note1, note2, note3])`

#### Pattern: "11 22" (two beats, each with two sixteenth notes)
- Input: `| 11 22 |`
- Beat structure: 2 beats, each with 2 subdivisions
- Each note duration: 1/16
- Expected: Two separate beam groups
  - Beat 1: notes 0-1 beamed together
  - Beat 2: notes 2-3 beamed together
- VexFlow: 
  - `new VF.Beam([note0, note1])`
  - `new VF.Beam([note2, note3])`

#### Pattern: "1-2" (tuplet - 3/2 tuplet)
- Input: `| 1-2 |`
- Beat structure: 1 beat with 3 subdivisions (tuplet)
- Expected: NO beaming (tuplets handle their own visual grouping)
- VexFlow: `new VF.Tuplet([note0, note1])` with no beam

#### Pattern: "1234" (four eighth notes in one beat)
- Input: `| 1234 |` with explicit eighth note durations
- Beat structure: 1 beat with 4 eighth notes
- Expected: All 4 eighth notes beamed together
- VexFlow: `new VF.Beam([note0, note1, note2, note3])`

### 3. Special Cases

#### Mixed Durations in a Beat
- If a beat contains both beamable and non-beamable notes, only beam consecutive beamable notes
- Example: "1 2 3" where 1 is a quarter note and 2,3 are eighth notes
  - Beam notes 2 and 3 together
  - Note 1 stands alone

#### Rests
- Rests break beam groups
- Example: "11-11" where - is a rest
  - First two notes beamed together
  - Last two notes beamed together
  - Rest in the middle breaks the beam

## Implementation Algorithm

```pseudocode
for each beat in beats:
    beat_beam_group = []
    
    for each note in beat.elements:
        if note.is_beamable() and not beat.is_tuplet:
            beat_beam_group.push(note_index)
        else:
            // Non-beamable note or tuplet
            if beat_beam_group.length >= 2:
                create_beam(beat_beam_group)
            beat_beam_group.clear()
    
    // End of beat - create beam for remaining notes
    if beat_beam_group.length >= 2:
        create_beam(beat_beam_group)
```

## Testing Requirements

The Playwright test should verify:
1. SVG contains `vf-beam` class when beamable notes are present
2. Correct number of beam elements for the pattern
3. Visual inspection shows proper beam rendering

### Test Cases
- `| 1111 |` - Should have 1 beam group with 4 notes
- `| SSSS |` - Should have 1 beam group with 4 notes  
- `| 11 22 |` - Should have 2 beam groups with 2 notes each
- `| 1234 |` - Should have 1 beam group with 4 notes (if eighth notes)
- `| 1-2 |` - Should have NO beams (tuplet)