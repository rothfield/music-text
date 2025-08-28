# IMPORTANT: Rhythm and FSM System - Critical LLM Reference

## IMPORTANT: ESSENTIAL READING FOR LLM INTERACTIONS

This document explains the rhythm/tuplet system that is **FUNDAMENTAL** to this music notation parser. LLMs typically lack understanding of these music notation implementation details.

**IMPORTANT**: This system was "already working in doremi-script V1" - the V2 system must match this logic exactly.

## IMPORTANT: Consolidation of Rhythm Knowledge

This file consolidates rhythm information from:
- LESSONS_LEARNED_V2.md (FSM architecture insights)
- VEXFLOW_V2_PLAN.md (VexFlow integration patterns)
- Previous debugging sessions and fixes

## Core FSM Rhythm Logic

### Tuplet Detection (Power of 2 Check)
```rust
let is_tuplet = beat.divisions > 1 && (beat.divisions & (beat.divisions - 1)) != 0;
```

**Examples:**
- divisions=2,4,8,16,32,64,128,256... → NOT tuplets (powers of 2)
- divisions=3,5,6,7,9,10,11,31,129... → ARE tuplets (not powers of 2)

### Tuplet Denominator Calculation
For tuplet X/Y, Y is the **largest power of 2 less than X**:
```rust
let mut tuplet_den = 1;
while tuplet_den * 2 < beat.divisions {
    tuplet_den *= 2;
}
```

**Examples:**
- 3 notes → 3/2 tuplet (triplet: 3 in place of 2)
- 5 notes → 5/4 tuplet (quintuplet: 5 in place of 4) 
- 31 notes → 31/16 tuplet (31 in place of 16)
- 129 notes → 129/64 tuplet (129 in place of 64)

## Beat Structure and Subdivisions

### FSM Beat Processing
When processing input like "1-2":
1. "1" creates beat with divisions=1, [Note1: subdivisions=1]
2. "-" extends Note1: divisions=2, [Note1: subdivisions=2]
3. "2" adds Note2: divisions=3, [Note1: subdivisions=2, Note2: subdivisions=1]

### Subdivision to Duration Mapping
**Within tuplets**, subdivisions map to **standard note durations** (not fractional calculations):

For 3-tuplet (divisions=3):
- subdivisions=1 → eighth note (8)
- subdivisions=2 → quarter note (4)
- subdivisions=3 → dotted quarter (4.)

For large tuplets (divisions=31):
- subdivisions=1 → sixty-fourth note (64)
- subdivisions=2 → thirty-second note (32)

## Key Files and Functions

### Core Rhythm Processing
- `src/rhythm_fsm_v2.rs` - V2 FSM that determines tuplets vs regular beats
- `src/rhythm.rs` - Shared rhythm/duration conversion utilities using fractional arithmetic

### Duration Conversion
- `src/lilypond_converter_v2.rs::calculate_tuplet_duration()` - Maps subdivisions to LilyPond durations
- `src/vexflow_converter_v2.rs::convert_tuplet_duration_to_vexflow_v2()` - Maps subdivisions to VexFlow durations

### Output Generation
- LilyPond: `\tuplet 3/2 { c4 d8 }` (quarter note + eighth note in triplet)
- VexFlow: `{"duration":"q"}` and `{"duration":"8"}` with `"ratio":[3,2]`

## Common LLM Mistakes to Avoid

### ❌ Wrong Thinking: "Fractional beat portions"
- "2/3 of a beat" → complex tied durations
- "1/3 of a beat" → more complex tied durations

### ✅ Correct Thinking: "Standard durations in tuplet context"  
- subdivisions=2 in 3-tuplet → quarter note
- subdivisions=1 in 3-tuplet → eighth note
- Tuplet bracket handles the "3 in place of 2" timing

### ❌ Wrong: Hardcoded tuplet denominators
```rust
match divisions {
    3 => 2, 5 => 4, 6 => 4, 7 => 4, // hardcoded mess
}
```

### ✅ Correct: Power-of-2 calculation
```rust  
let mut tuplet_den = 1;
while tuplet_den * 2 < beat.divisions {
    tuplet_den *= 2;
}
```

## Testing Examples

### "1-2" Input
- FSM: divisions=3, [subdivisions=2, subdivisions=1] → 3/2 tuplet
- LilyPond: `\tuplet 3/2 { c4 d8 }`
- VexFlow: quarter note + eighth note with ratio [3,2]

### "1111111111111111111111111111111" (31 ones)
- FSM: divisions=31, all subdivisions=1 → 31/16 tuplet  
- LilyPond: `\tuplet 31/16 { c64 c64 c64... }` (31 sixty-fourth notes)
- VexFlow: All sixty-fourth notes with ratio [31,16]

## IMPORTANT: Architectural Principle

**IMPORTANT**: The FSM determines rhythm structure. The converters map subdivisions to appropriate visual durations. Always use fractional arithmetic, never floating point.

**IMPORTANT**: This system was "already working in doremi-script V1" - the V2 system must match this logic exactly.

## IMPORTANT: Key Insights from LESSONS_LEARNED_V2.md

### FSM Architecture (Section 4-5)
- **Clean room implementation beats adaptation** - V2 FSM started fresh instead of adapting V1 complexity
- **Musical rhythm is fundamentally about fractions** - don't overthink with complex state machines
- Simple counting + fraction reduction handles most cases correctly

### Duration Calculation Simplicity
```rust
fn finish_current_beat(&mut self) {
    if let Some(beat) = self.current_beat.take() {
        for elem_with_subdivisions in beat.elements {
            let (reduced_num, reduced_denom) = reduce_fraction(
                elem_with_subdivisions.subdivisions, 
                beat.total_divisions
            );
            // Emit element with duration (reduced_num, reduced_denom)
        }
    }
}
```

## IMPORTANT: Complete Workflow Example

### CRITICAL LLM MISTAKES TO AVOID

❌ **WRONG**: Treating dashes as separate elements
- "1-2-3" ≠ three separate notes
- "1-2-3" = one extended note (1) + note (2) + note (3)

❌ **WRONG**: Ignoring FSM subdivision logic  
- Don't just count notes and guess tuplet ratios
- Must use divisions & subdivisions from FSM output

❌ **WRONG**: Treating spaces as rests
- Spaces are beat separators, not musical rests
- "-" at start of beat = rest element

### Converting "1-2" to Western Notation (Step by Step)

**Input**: `"1-2"`
**Expected Output**: 3/2 tuplet with C quarter note + D eighth note

**Step 1: Parse**
```rust
// Entry point: src/lib.rs::parse_notation()
let input = "1-2";
let document = parse_notation(input)?;
```

**Step 2: Extract Elements** 
```rust  
// Elements: [Note{N1}, Dash, Note{N2}]
// N1 = PitchCode for "1" (maps to C)  
// N2 = PitchCode for "2" (maps to D)
```

**Step 3: FSM Processing**
```rust
// src/rhythm_fsm_v2.rs
let fsm_output = process_elements(&elements);
// Result: BeatV2 { divisions: 3, elements: [
//   ElementV2 { element: Note{N1}, subdivisions: 2 },
//   ElementV2 { element: Note{N2}, subdivisions: 1 }
// ]}
```

**Step 4: Convert to Western Notation**
```rust
// LilyPond: src/lilypond_converter_v2.rs
let lilypond = convert_to_lilypond(&fsm_output);
// Result: "\tuplet 3/2 { c4 d8 }"

// VexFlow: src/vexflow_converter_v2.rs  
let vexflow = convert_to_vexflow(&fsm_output);
// Result: [Note{keys:["c/4"], duration:"q"}, Note{keys:["d/4"], duration:"8"}]
```

**IMPORTANT Pitch Mapping Reference:**
- "1" → N1 → C (western)
- "2" → N2 → D (western) 
- "3" → N3 → E (western)
- etc.

## IMPORTANT: Final Reminder

**This rhythm system is standard music theory, not rocket science. The complexity comes from implementation details that LLMs typically haven't encountered in training data. When in doubt, refer to this document and the working V1 system.**

**For fresh LLM sessions**: Follow the complete workflow example above, then dive into the specific rhythm/tuplet logic documented in this file.

## IMPORTANT: Complex Example - "1-2-3 -4#"

### CORRECT Analysis:
```
Input: "1-2-3 -4#"
Elements: [Note{1}, Dash, Note{2}, Dash, Note{3}, Space, Dash, Note{4#}]

Beat 1: "1-2-3" 
- Note{1}: starts with subdivisions=1
- Dash: extends Note{1} to subdivisions=2  
- Note{2}: adds element with subdivisions=1
- Dash: extends Note{2} to subdivisions=2
- Note{3}: adds element with subdivisions=1
- Result: divisions=5, [Note{1,sub=2}, Note{2,sub=2}, Note{3,sub=1}]
- 5 is not power of 2 → 5/4 tuplet

Beat 2: "-4#"
- Dash: creates Rest with subdivisions=1
- Note{4#}: adds element with subdivisions=1  
- Result: divisions=2, [Rest{sub=1}, Note{4#,sub=1}]
- 2 is power of 2 → not a tuplet (regular beat)

Western Output:
Beat 1: \tuplet 5/4 { c4 d4 e8 }  
Beat 2: r4 fs4
```

### CRITICAL: Duration Calculation with FRACTIONAL ARITHMETIC

**ALWAYS USE FRACTIONAL ARITHMETIC - NEVER DECIMALS!**

For 5-tuplet with beat duration = 1/4:

**Step 1: Calculate each note's fraction of the beat**
- Note 1: subdivisions=2, divisions=5 → gets 2/5 of the beat
- Note 2: subdivisions=2, divisions=5 → gets 2/5 of the beat  
- Note 3: subdivisions=1, divisions=5 → gets 1/5 of the beat

**Step 2: Multiply by beat duration (1/4) using FRACTIONS**
- Note 1: (2/5) × (1/4) = 2/20 = 1/10 duration
- Note 2: (2/5) × (1/4) = 2/20 = 1/10 duration
- Note 3: (1/5) × (1/4) = 1/20 duration

**Step 3: Map to visual durations using tuplet logic**
- Use the subdivision-to-duration mapping from code
- NOT the exact fractional calculation above
- The fractions inform the relative proportions within the tuplet

**CRITICAL: The exact fractional math is for internal calculation - the visual output uses the tuplet duration mapping logic from the code!**

### WHY THIS IS COMPLEX:
- **5-tuplet**: 5 notes in place of 4 (next lower power of 2)  
- **Different subdivisions**: Note 1&2 get 2 subdivisions each, Note 3 gets 1
- **Fractional arithmetic**: Must use fractions throughout, never decimals
- **Rest handling**: Leading dash in beat creates rest element
- **Beat separation**: Space creates new beat, not rest

**IMPORTANT: LLMs often get the duration calculation wrong - always use the exact fractional arithmetic shown above!**