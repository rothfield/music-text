# Claude Code Assistant Memory

## Important Project Guidelines

### Development Workflow
- **Testing Requirement**: The job is not done until the web UI is actually tested
- Always verify fixes work in the browser interface, not just in backend code
- Check console output and visual rendering to confirm solutions

### V2 Hybrid Architecture

#### **Single Unified Codebase**
- **One Rust Codebase**: `src/` contains all logic - no separate WASM codebase
- **Dual Compilation**: Same code compiles to both native binary and WASM
- **WASM Generation**: `wasm-pack build --target web --out-dir webapp/pkg` creates client-side artifacts

#### **Execution Environments**
- **CLI**: Native Rust binary for command-line processing
- **Client-Side Web**: WASM (`notation_parser.js` + `.wasm`) runs V2 parser + VexFlow rendering in browser
- **Server-Side Web**: Node.js server for LilyPond SVG generation (high-quality output)

#### **Data Flow**
```
Single Rust Codebase → {
    CLI: Native binary execution
    WASM: Browser execution (V2 parser + VexFlow)
    Server: LilyPond generation via CLI calls
}
```

#### **Web Interface Architecture**
- **Port 3000**: Express server at http://localhost:3000
- **Client-Side**: WASM parser + live VexFlow rendering (instant feedback)
- **Server-Side**: LilyPond SVG generation (professional output)
- **Hybrid Benefits**: Fast client interaction + high-quality server rendering

## IMPORTANT: Notation Systems Overview

### Supported Input Notation Systems
This parser supports multiple musical notation input systems:

1. **Western Notation**: C D E F G A B (standard western notes)
2. **Sargam Notation**: S R G M P D N (Indian classical music)
3. **Number Notation**: 1 2 3 4 5 6 7 (numeric system, most common in examples)

### Key Mapping (Number → Western → Sargam)
```
1 → C → S (Do)
2 → D → R (Re)  
3 → E → G (Mi)
4 → F → M (Fa)
5 → G → P (Sol)
6 → A → D (La)
7 → B → N (Ti)
```

### Internal Representation: PitchCode Enum
All pitches are normalized to internal `PitchCode` enum:
```rust
pub enum PitchCode {
    N1, N2, N3, N4, N5, N6, N7  // Normalized representation
}
```

### Conversion Flow
```
Input: "1-2" → Parse: [Note{N1}, Dash, Note{N2}] → FSM → Output: "C4 D8 tuplet"
Input: "S-R" → Parse: [Note{N1}, Dash, Note{N2}] → FSM → Output: "C4 D8 tuplet"  
Input: "C-D" → Parse: [Note{N1}, Dash, Note{N2}] → FSM → Output: "C4 D8 tuplet"
```

**IMPORTANT**: All three input systems produce identical internal representation and output!

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

## IMPORTANT IMPORTANT IMPORTANT: Rhythm System Understanding

⚠️ **IMPORTANT: READ RHYTHM_SYSTEM.md FIRST** ⚠️  

The rhythm/tuplet system is **IMPORTANT AND ALIEN TO LLMs** and requires careful study. See `RHYTHM_SYSTEM.md` for complete documentation.

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
- `wasm-pack build --target web --out-dir webapp/pkg` - Build WASM for web UI
- `cd webapp && node server.js` - Start web server (port 3000)
- Test tuplets like "1-2" in the web interface
- Verify both VexFlow rendering and LilyPond source output
- **Use Playwright for automated browser testing when needed** - `npx playwright test`

### Current Status
- ✅ Fixed V2 LilyPond tuplet generation to use standard durations
- ✅ V2 FSM correctly identifies tuplets using power-of-2 check
- 🔄 VexFlow converter may still need tuplet duration fixes (uniform vs proportional durations)
- 🔄 Need to test "1-2" in web UI to verify both outputs work correctly