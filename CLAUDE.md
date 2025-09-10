# Claude Code Assistant Memory

## 🚨 STARTUP WORKFLOW
**When Claude starts a new session:**
1. Read `TODO_LIST.md` 
2. Ask if user wants to work on the top item
3. If yes, use TodoWrite to create todo list and begin work

---

## Task Management

**TODO List**: See `TODO_LIST.md` for all current tasks. When the user says "add a todo", add it to `TODO_LIST.md`.

## Important Project Guidelines


### ⚠️ CRITICAL: NO INLINE OCTAVE NOTATION
**IMPORTANT**: This system uses **spatial octave markers** only. There is NO inline octave notation.
- ❌ **WRONG**: "1." as inline lower octave notation  
- ✅ **CORRECT**: "1" with dot marker below in separate annotation line
- **Grammar Rule**: `number_pitch = { ASCII_DIGIT ~ flat_or_sharp? }` - NO dots allowed
- **Octave System**: All octave information comes from spatial markers (upper/lower annotation lines)

### 🚨 CRITICAL: PARSER REQUIRES BARLINES FOR VALID INPUT
**IMPORTANT**: The current parser WILL NOT parse musical input without barlines.
- ❌ **FAILS**: `"1"`, `"SRG"`, `"1 2 3"` - all fail with "expected barline"
- ✅ **WORKS**: `"|1"`, `"|SRG"`, `"|1 2 3"` - barline prefix makes input parseable
- **Grammar Constraint**: Current grammar expects barlines to separate musical content
- **Testing Requirement**: Always prefix test inputs with `|` for successful parsing

### 🚨 PEST GRAMMAR ISSUE IDENTIFIED: "1\n." PARSING FAILURE  
**Status**: Architecture refactor planned  
**Problem**: Current grammar cannot disambiguate upper vs lower octave markers based on position  
**Root Cause**: Parser tries to classify line types during parsing instead of post-processing  

**Current Broken Behavior**:
```
Input: "1\n."
Expected: content_line("1") + lower_line(".")  
Actual: Fails with "unexpected segment" or "expected number_upper_line_item"
```

**Solution**: Multi-phase parsing with specialized grammars
- **Phase 1**: Parse structure with position-aware grammars (`upper_grammar` vs `lower_grammar`)  
- **Phase 2**: Classify parsed content into final AST types
- **Architecture**: `RawStave` → classification → `Stave`

**Implementation Plans**:
- `TECH_NOTE_GRAMMAR_REFACTOR.md` - High-level architectural proposal
- `DETAILED_CODE_PLAN_GRAMMAR_REFACTOR.md` - Code-level implementation plan  
- `GRAMMAR_REFACTOR_CRITIQUE.md` - Expert validation of approach

**Timeline**: Major refactor required - estimated 2-4 weeks implementation

### Development Workflow
- **Testing Requirement**: The job is not done until the web UI is actually tested
- Always verify fixes work in the browser interface, not just in backend code
- Check console output and visual rendering to confirm solutions
- **Use Playwright as primary testing tool**: Run `npx playwright test` for automated browser testing
- **Playwright for general browser automation**: Use Playwright for any browser interaction, debugging, or testing needs

### V2 Hybrid Architecture

#### **Single Unified Codebase**
- **One Rust Codebase**: `src/` contains all logic
- **Native Compilation**: Rust code compiles to native binary for CLI and web server

#### **WYSIWYG Editor Planning** 🚧
- **Status**: Architecture proposal under consideration (see `WYSIWYG_ARCHITECTURE_PROPOSAL.md`)
- **Goal**: Add visual slur editing while preserving existing lexer/parser pipeline
- **Approach**: Visual editor → spatial format generator → existing `unified_parser()`
- **Key insight**: Convert visual slurs to 2-line spatial format that existing lexer expects

#### **Execution Environments**
- **CLI**: Native Rust binary for command-line processing
- **Web Server**: Rust Axum server provides API endpoints for parsing
- **Web Client**: JavaScript frontend calls Rust API for parsing and rendering

#### **Data Flow**
```
Single Rust Codebase → {
    CLI: Native binary execution
    Web Server: Axum API endpoints
    Web Client: API calls for parsing and rendering
}
```

#### **Web Interface Architecture**
- **Port 3000**: Rust Axum server at http://localhost:3000
- **Client-Side**: JavaScript frontend with API calls for parsing
- **Server-Side**: Rust API endpoints + LilyPond SVG generation (professional output)
- **Benefits**: Clean API separation + high-quality server rendering

## 🎼 CRITICAL: TONIC-BASED MOVABLE-DO SYSTEM

⚠️ **THIS IS A TONIC-CENTERED SYSTEM, NOT A KEY SIGNATURE SYSTEM** ⚠️

### Core Philosophy: TONIC as Reference Pitch
This system treats all pitches as **relative to a tonic**, not as absolute pitches:

- **Scale degree 1 = TONIC** (whatever pitch is declared as tonic)
- **Scale degree 2 = second degree above tonic** 
- **Scale degree 7 = seventh degree above tonic**
- **Tonic can be ANY pitch** (C, D, F#, Bb, etc.)

### TONIC Examples:
```
key: C  →  1 = C, 2 = D, 3 = E, 4 = F, 5 = G, 6 = A, 7 = B
key: D  →  1 = D, 2 = E, 3 = F#, 4 = G, 5 = A, 6 = B, 7 = C#  
key: G  →  1 = G, 2 = A, 3 = B, 4 = C, 5 = D, 6 = E, 7 = F#
key: Bb →  1 = Bb, 2 = C, 3 = D, 4 = Eb, 5 = F, 6 = G, 7 = A
```

**CRITICAL**: This is **NOT** about major/minor modes or traditional key signatures. It's purely about **TONIC as the reference pitch**.

### ❌ WRONG Thinking: "D Major Scale"  
### ✅ CORRECT Thinking: "D as Tonic"

When `key: D` and input `1` → output `D` because **1 = tonic = D**

## IMPORTANT: Notation Systems Overview

### Supported Input Notation Systems
This parser supports multiple musical notation input systems, **ALL work with tonic transposition**:

1. **Western Notation**: C D E F G A B (standard western notes)
2. **Sargam Notation**: S R G M P D N (Indian classical music)  
3. **Number Notation**: 1 2 3 4 5 6 7 (numeric system, most common in examples)

### Default Mapping (when NO tonic specified - defaults to C/S/1 as tonic)
```
Number → Western → Sargam
1      → C       → S     (tonic/Do/Sa)
2      → D       → R     (second degree/Re)  
3      → E       → G     (third degree/Mi/Ga)
4      → F       → M     (fourth degree/Fa/Ma)
5      → G       → P     (fifth degree/Sol/Pa)
6      → A       → D     (sixth degree/La/Dha)
7      → B       → N     (seventh degree/Ti/Ni)
```

### Internal Representation: Degree Enum
All pitches are normalized to internal `Degree` enum (scale degrees):
```rust
pub enum Degree {
    N1, N2, N3, N4, N5, N6, N7  // Scale degrees 1-7
    N1s, N1b, N2s, N2b, ...    // With sharps and flats
}
```

### Tonic Transposition Flow
```
Input: "key: D" → Sets tonic to D (N2)
Input: "1-2"    → Parse: [Tonic(N2), Note{N1}, Dash, Note{N2}] 
                → FSM processes with tonic context
                → Output: "d4 e8" (D-E, not C-D!)
```

**CRITICAL**: Scale degrees are **RELATIVE TO TONIC**, not absolute pitches!

## CRITICAL CRITICAL CRITICAL: Dash (-) Behavior

⚠️ **CRITICAL: DASH IS THE EXTENDER SYMBOL** ⚠️

The dash (-) character has **DUAL BEHAVIOR** that is essential to understand:

### Primary Function: EXTENSION
- **Extends notes within beats, across beats, and through barlines**
- **Creates tied notes when extending a previous pitch**
- **Duration extension**: Each dash adds one subdivision to the previous note

### Secondary Function: REST
- **Only when there is NO previous note to extend**
- **Only when a breath mark (') has occurred, breaking the extension chain**

### Critical Examples:

**Extension Examples:**
- `S- -S` → S extended (2 subdivisions) tied to S (1 subdivision) = `c4~ c8 c8`
- `S--R` → S extended (3 subdivisions) + R (1 subdivision) 
- `S-|-R` → S extended across barline, tied to R

**Rest Examples:**
- `-S` → rest + S (dash creates rest since no previous note)
- `S' -R` → S + breath + rest + R (breath breaks extension chain)

**CRITICAL RULE:** Dash ALWAYS tries to extend the previous note first. Only creates a rest if:
1. No previous note exists, OR
2. A breath mark (') has broken the extension chain

**CRITICAL FOR FSMS:** The FSM must track the "extension chain" state and distinguish between:
- Dash as extension (tied note creation)  
- Dash as rest (when extension chain is broken)

## IMPORTANT: Slur Notation

⚠️ **SLUR NOTATION CHANGE** ⚠️

- **Parentheses () are NO LONGER supported for slurs**
- **Use underscores (_____) above the notes for slurs**
- Example: `1_2_3` creates a slur over notes 1, 2, and 3
- Multiple underscores can span multiple notes within a slur
- This change prevents confusion with other notation elements

## IMPORTANT IMPORTANT IMPORTANT: Rhythm System Understanding

⚠️ **CRITICAL: RHYTHM AND FSM SYSTEM - ESSENTIAL READING** ⚠️

**IMPORTANT: This is NOT rocket science** - it's standard music notation, but LLMs lack this domain knowledge.

**IMPORTANT Quick Reference:**
- Tuplet detection: `beat.divisions` not power of 2
- Tuplet denominators: largest power of 2 less than divisions  
- Duration mapping: subdivisions → standard note durations (NOT fractional beat portions)
- "1-2" → 3/2 tuplet with quarter + eighth notes
- **CRITICAL**: Always use fractional arithmetic, never floating point - LLMs fail here!

**IMPORTANT Key Files:**
- `src/rhythm_fsm_v2.rs` - FSM tuplet detection logic
- `src/lilypond_converter_v2.rs` - LilyPond tuplet duration mapping  
- `src/vexflow_converter_v2.rs` - VexFlow tuplet duration mapping

**IMPORTANT: When working on rhythm/FSM issues, ALWAYS read RHYTHM_SYSTEM.md first!**

## CRITICAL CRITICAL CRITICAL: Tuplet Duration Conversion Rule

**THE SIMPLE RULE FOR TUPLET DURATIONS:**

For any tuplet with N divisions (where N is not a power of 2):
1. **Find the next lower power of 2** (call it P)
2. **Calculate durations as if divisions = P**
3. **Wrap in tuplet N/P**

**Example: "1-2-3"**
- Subdivisions: 2+2+1 = 5 total
- 5 is not power of 2 → 5-tuplet  
- Next lower power of 2 = 4
- **Calculate as if divisions=4:**
  - Each unit = 1/4 ÷ 4 = 1/16
  - Note 1: 2×(1/16) = 1/8 → eighth note
  - Note 2: 2×(1/16) = 1/8 → eighth note
  - Note 3: 1×(1/16) = 1/16 → sixteenth note
- **Result:** `\tuplet 5/4 { c8 d8 e16 }`

**CRITICAL:** Don't overthink this! Just convert as if it's the power-of-2 division, then wrap in tuplet bracket!

**CRITICAL:** This is the ONLY correct way to calculate tuplet durations - ignore any other complex methods!

### Recent Issues Fixed
1. **Slur positioning** - Fixed spatial analysis to prevent multiple slur end markers
2. **Tie logic** - Only create ties between notes of same pitch (correct musical definition)  
3. **VexFlow crash** - Fixed slur indexing by separating notes from barlines in array handling
4. **Tuplet rhythm parsing** - Fixed V2 LilyPond converter to generate proper `\tuplet` notation with standard durations

### Key Files
- `src/rhythm_fsm_v2.rs` - V2 rhythm FSM that determines tuplets vs regular beats  
- `src/lilypond_converter_v2.rs` - V2 LilyPond converter with tuplet support
- `src/vexflow_converter_v2.rs` - V2 VexFlow converter (may need tuplet duration fixes)
- `src/rhythm.rs` - Shared rhythm/duration conversion utilities
- `webapp/` - Web UI for testing both VexFlow and LilyPond output

### Testing Commands  
- `cargo build --release` - Build backend
- `cd webapp && node server.js` - Start web server (port 3000)
- **PRIMARY TESTING**: `npx playwright test` - Run automated browser tests
- **Playwright Test Development**: Use Playwright for all browser testing needs:
  - `npx playwright test --headed` - Run tests with visible browser
  - `npx playwright test --debug` - Debug tests interactively
  - `npx playwright codegen` - Generate test code by recording browser interactions
- Test tuplets like "1-2" in the web interface
- Verify both VexFlow rendering and LilyPond source output

### Current Status
- ✅ Fixed V2 LilyPond tuplet generation to use standard durations
- ✅ V2 FSM correctly identifies tuplets using power-of-2 check
- 🔄 VexFlow converter may still need tuplet duration fixes (uniform vs proportional durations)
- 🔄 Need to test "1-2" in web UI to verify both outputs work correctly

# IMPORTANT: Rhythm and FSM System - Critical LLM Reference

## IMPORTANT: ESSENTIAL READING FOR LLM INTERACTIONS

This document explains the rhythm/tuplet system that is **FUNDAMENTAL** to this music music-text. LLMs typically lack understanding of these music notation implementation details.

**IMPORTANT**: This system was "already working in music-text V1" - the V2 system must match this logic exactly.

## IMPORTANT: Consolidation of Rhythm Knowledge

This file consolidates rhythm information from:
- LESSONS_LEARNED_V2.md (FSM architecture insights)
- VEXFLOW_V2_PLAN.md (VexFlow integration patterns)
- Previous debugging sessions and fixes

## CRITICAL: Fundamental Music Sound Model - Struck/Unstruck Dichotomy

**IMPORTANT**: This is the foundational musical model underlying the entire system.

### Core Sound Types
1. **STRUCK** = Notes (attack/onset) - musical sounds that begin with an attack
2. **UNSTRUCK** = Rests (silence) - periods of no musical sound

### Duration System
- Each sound (struck or unstruck) continues for a certain number of **subdivisions**
- Subdivisions determine the duration of that sound event
- Multiple consecutive subdivisions = longer duration of the same sound type

### Extension and Continuation
- **Within Beat Extension**: Dashes (-) extend the current sound for more subdivisions
- **Cross-Beat Ties**: When a sound extends from one beat into the next beat

### CRITICAL: True Rests vs Extensions - Dash Behavior

**IMPORTANT**: Dashes have dual behavior that is essential to understand:

#### Primary Function: EXTENSION
- **Extends previous struck/unstruck sounds within beats, across beats, and through barlines**
- **Creates longer durations by adding subdivisions to the previous sound**
- **Each dash adds one subdivision to the previous sound event**

#### Secondary Function: TRUE REST (Unstruck Sound)
- **Only when there is NO previous sound to extend**
- **Only when a breath mark (') has occurred, breaking the extension chain**

#### Critical Examples:

**EXTENSION Examples:**
- `1-` → Note "1" extended by 1 subdivision (quarter note becomes half note)
- `1--` → Note "1" extended by 2 subdivisions (quarter becomes dotted half)
- `1--1` → Note "1" extended by 2 subdivisions + new Note "1" (dotted quarter + eighth)

**TRUE REST Examples:**
- `-1` → TRUE REST (1/8) + Note "1" (1/8) - dash creates rest since no previous sound
- `1' -1` → Note "1" + breath + TRUE REST + Note "1" - breath breaks extension chain

#### The Extension Chain
**CRITICAL RULE**: The FSM tracks the "extension chain" state:
- Dashes ALWAYS try to extend the previous sound first
- Only creates TRUE REST if:
  1. No previous sound exists (start of beat), OR
  2. A breath mark (') has broken the extension chain

**Example Analysis:**
- `-1` → divisions=2, [Rest{sub=1}, Note{sub=1}] = 1/8 rest + 1/8 C
- `1- '-1` → Beat 1: [Note{sub=2}] = 1/4 C, Beat 2: [Rest{sub=1}, Note{sub=1}] = 1/8 rest + 1/8 C

### Musical Examples Using Correct Dash Logic

#### `1--1` (Extended Note + New Note)
- Input: [Note{1}, Dash, Dash, Note{1}]
- FSM: Beat with divisions=4, [Note{sub=3}, Note{sub=1}] 
- Sound 1: STRUCK (C) for 3 subdivisions → dotted quarter note
- Sound 2: STRUCK (C) for 1 subdivision → eighth note
- Result: Two separate C attacks with different durations (within 3/2 tuplet)

#### `-1-` (True Rest + Extended Note)
- Input: [Dash, Note{1}, Dash]
- FSM: Beat with divisions=3, [Rest{sub=1}, Note{sub=2}]
- Sound 1: UNSTRUCK (rest) for 1 subdivision → eighth rest
- Sound 2: STRUCK (C) for 2 subdivisions → quarter note
- Result: Eighth rest followed by quarter note (within 3/2 tuplet)

#### `1- '-1` (Extended Note + Breath + Rest + Note)
- Input: [Note{1}, Dash, Space, Breath, Dash, Note{1}]
- FSM: Beat 1: divisions=2, [Note{sub=2}]; Beat 2: divisions=2, [Rest{sub=1}, Note{sub=1}]
- Beat 1: STRUCK (C) for 2 subdivisions → half note
- Beat 2: UNSTRUCK (rest) for 1 subdivision + STRUCK (C) for 1 subdivision → quarter rest + quarter note
- Result: Half note C, quarter rest, quarter note C

### Critical Implementation Notes
- Each `BeatElement` represents one sound event (struck or unstruck)
- The `subdivisions` field determines how long that sound continues
- Ties occur when sounds extend across beat boundaries
- Both struck sounds (notes) AND unstruck sounds (rests) can be tied

**IMPORTANT**: This struck/unstruck model is fundamental to understanding rhythm, ties, and duration in this system. Traditional music notation concepts of "tied notes" are just one case of this more general sound continuation model.

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

**IMPORTANT**: This system was "already working in music-text V1" - the V2 system must match this logic exactly.

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
- -release if you are using --release then that is a mistake!!! use make build
- NO ENHARMONICS
- # to compile, use Makefile