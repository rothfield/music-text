# Technical Note: Tuplet Implementation

## Overview

This document describes how tuplets (triplets, quintuplets, septuplets, etc.) are handled in the notation parser system.

## Key Principle

**Tuplets use standard note durations with ratio compression signaling.**

When you have a tuplet (n notes where n is not a power of 2):
1. **Write the notes** using the next power of 2 as the denominator 
2. **Add a bracket with "n"** over them to show the actual timing ratio

## Examples

### Triplet (3 notes)
- **Input**: `| SRG |` (3 notes packed together)
- **Notation**: Three 1/16 notes with "3" bracket
- **Meaning**: Play 3 sixteenth notes in the time of 2 sixteenth notes
- **Ratio**: [3,2]

### Quintuplet (5 notes) 
- **Input**: `| SRGMP |` (5 notes packed together)
- **Notation**: Five 1/32 notes with "5" bracket  
- **Meaning**: Play 5 thirty-second notes in the time of 4 thirty-second notes
- **Ratio**: [5,4]

### Septuplet (7 notes)
- **Input**: `| SRGMPDN |` (7 notes packed together)
- **Notation**: Seven 1/32 notes with "7" bracket
- **Meaning**: Play 7 thirty-second notes in the time of 4 thirty-second notes  
- **Ratio**: [7,4]

## Implementation Architecture

### 1. FSM Detection (Rust)
```rust
// Detect non-power-of-2 beat divisions
let is_tuplet = total_subdivisions > 1 && (total_subdivisions & (total_subdivisions - 1)) != 0;

// Calculate ratio 
let tuplet_ratio = match total_subdivisions {
    3 => (3, 2),  // Triplet: 3 in time of 2
    5 => (5, 4),  // Quintuplet: 5 in time of 4  
    7 => (7, 4),  // Septuplet: 7 in time of 4
    // etc.
};
```

### 2. Duration Calculation (Rust)
Uses **next power of 2** for note durations:
```rust
fn next_power_of_two(n: usize) -> usize {
    let mut power = 1;
    while power < n {
        power *= 2;
    }
    power
}

// For tuplets:
let next_power_of_2 = next_power_of_two(beat.divisions);
let duration = format!("1/{}", next_power_of_2 * 4);
```

### 3. VexFlow JSON Output (Rust)
```json
{
  "type": "Tuplet",
  "notes": [
    {"type": "Note", "duration": "16", "keys": ["c/4"]},
    {"type": "Note", "duration": "16", "keys": ["d/4"]}, 
    {"type": "Note", "duration": "16", "keys": ["e/4"]}
  ],
  "ratio": [3, 2]
}
```

### 4. Frontend Rendering (JavaScript)
```javascript
// Handle Tuplet objects from FSM
if (element.type === 'Tuplet') {
    const tupletNotes = element.notes.map(noteData => {
        // Create VexFlow notes with standard durations
        return new Vex.Flow.StaveNote({
            duration: noteData.duration // e.g., "16" for sixteenth note
        });
    });
    
    // Create tuplet bracket with ratio
    const tuplet = new Vex.Flow.Tuplet(tupletNotes);
    
    // Create beaming within tuplet
    if (tupletNotes.length >= 2) {
        const beam = new Beam(tupletNotes);
    }
}
```

## Mathematical Logic

### Duration Calculation Examples

**Triplet (3 notes):**
- Next power of 2: 4
- Duration: 1/(4*4) = 1/16 
- Result: Three 1/16 notes with [3,2] ratio

**Quintuplet (5 notes):**  
- Next power of 2: 8
- Duration: 1/(8*4) = 1/32
- Result: Five 1/32 notes with [5,4] ratio

**Septuplet (7 notes):**
- Next power of 2: 8  
- Duration: 1/(8*4) = 1/32
- Result: Seven 1/32 notes with [7,4] ratio

## Visual Result

The notation shows:
1. **Standard note durations** (1/8, 1/16, etc.) 
2. **Beaming** connecting the notes within the tuplet
3. **Bracket with number** (3, 5, 7) showing the actual ratio
4. **Proper spacing** and alignment

This creates readable music notation that follows standard conventions while accurately representing the spatial rhythmic relationships from the input.

## Key Design Decision

**Why use standard durations + ratio instead of fractional durations?**

Using `1/12` durations for triplets would be mathematically correct but visually confusing to musicians. Standard music notation uses familiar note values (1/8, 1/16) with tuplet brackets to indicate timing modifications. This approach:

- ✅ Follows standard music notation conventions
- ✅ Is readable by musicians  
- ✅ Works with standard music notation software
- ✅ Preserves the spatial relationship information from the original input

The tuplet bracket with ratio provides the timing information while keeping the visual representation familiar and clear.