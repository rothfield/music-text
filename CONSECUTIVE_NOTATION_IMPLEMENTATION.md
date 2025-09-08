# Consecutive Notation Implementation

**Date**: 2025-09-08  
**Status**: Implemented and tested  

## Overview

This document describes the implementation of consecutive notation detection for single-line musical input without requiring barlines. The feature enables automatic detection and parsing of consecutive musical characters like "SRG", "123", or "CDE" as valid musical notation.

## Problem Statement

### Original Issue
The music-text parser required barlines ("|") to identify musical content. Input like "SRG" would fail with "expected barline" errors, even though it represents valid Sargam notation (Sa-Re-Ga).

### Grammar Ambiguity Challenge
The PEST grammar uses ordered choice rules where ambiguous characters like 'G' are parsed as the first matching rule:
- 'G' in input "SRmG" was parsed as `western_pitch` (N5/G) instead of `sargam_pitch` (N3/Ga → E)
- This created mixed notation systems within a single musical line
- The user requirement: **SRGmPDN → 1234567** (Sargam to scale degrees)

## Solution Architecture

### Two-Phase Parsing Strategy
1. **Consecutive Detection First**: Check for 3+ consecutive musical characters from the same system
2. **Grammar Parsing Fallback**: Use normal PEST grammar if consecutive detection fails

### Key Design Principles
- **System Consistency**: All characters in consecutive input must belong to the same musical system
- **Minimum Length**: Require 3+ characters to avoid false positives
- **Single-Line Only**: Only process single-line input (ignore multi-line content)
- **Spaced Input Exclusion**: "S R G" (with spaces) treated as plain text, "SRG" (consecutive) as music

## Implementation Details

### 1. Modified Parse Flow (`src/document/mod.rs`)

**Before** (fallback approach):
```rust
pub fn parse_document(input: &str) -> Result<Document, String> {
    let mut document = build_document(input)?;  // Try grammar first
    if document.staves.is_empty() {             // Only if completely failed
        if let Some(stave) = try_create_consecutive_stave(input) {
            document.staves.push(stave);
        }
    }
    Ok(document)
}
```

**After** (priority approach):
```rust
pub fn parse_document(input: &str) -> Result<Document, String> {
    // Step 1: Check consecutive notation first (higher priority)
    if let Some(stave) = try_create_consecutive_stave(input) {
        let document_source = model::Source {
            value: input.to_string(),
            position: model::Position { line: 1, column: 1 },
        };
        return Ok(Document { 
            staves: vec![stave], 
            source: document_source 
        });
    }
    
    // Step 2: Try normal parsing if consecutive detection fails
    let document = build_document(input)?;
    Ok(document)
}
```

**Rationale**: The priority approach ensures that consecutive patterns are detected before grammar parsing can create mixed notation systems due to ordered choice ambiguity.

### 2. Consecutive Detection Logic

#### System Detection (`detect_system_from_consecutive`)
```rust
fn detect_system_from_consecutive(chars: &[char]) -> Option<NotationSystem> {
    // All must be numbers
    if chars.iter().all(|&c| matches!(c, '1'..='7')) {
        return Some(NotationSystem::Number);
    }
    
    // All must be Sargam (mixed case allowed)
    if chars.iter().all(|&c| matches!(c, 
        'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |           // Sargam upper
        's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'             // Sargam lower
    )) {
        return Some(NotationSystem::Sargam);
    }
    
    // All must be Western
    if chars.iter().all(|&c| matches!(c, 'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B')) {
        return Some(NotationSystem::Western);
    }
    
    None // Mixed systems or unrecognized
}
```

#### Character to Pitch Mapping (`char_to_pitchcode`)
```rust
NotationSystem::Sargam => match ch {
    'S' | 's' => Some(PitchCode::N1), // Sa
    'R' | 'r' => Some(PitchCode::N2), // Re  
    'G' | 'g' => Some(PitchCode::N3), // Ga  ← Key fix: N3 not N5
    'M' | 'm' => Some(PitchCode::N4), // Ma
    'P' | 'p' => Some(PitchCode::N5), // Pa
    'D' | 'd' => Some(PitchCode::N6), // Dha
    'N' | 'n' => Some(PitchCode::N7), // Ni
    _ => None,
}
```

### 3. Input Trimming Fix

**Problem**: Web interface sent `"SRG\n"` with trailing newlines, causing detection to fail.

**Solution**: Trim input before checking for newlines.
```rust
fn try_create_consecutive_stave(input: &str) -> Option<Stave> {
    let trimmed_input = input.trim();
    if trimmed_input.contains('\n') {
        return None;  // Multi-line content
    }
    // Use trimmed_input for all processing...
}
```

## Test Cases and Results

### ✅ Valid Consecutive Notation
| Input | System | Output | Notes |
|-------|--------|---------|-------|
| `"SRG"` | Sargam | `\version "2.24.0" { c4 d4 e4 }` | S→C, R→D, G→E |
| `"SRmG"` | Sargam | `\version "2.24.0" { c4 d4 f4 e4 }` | Fixed G→E (was G→G) |
| `"123"` | Number | `\version "2.24.0" { c4 d4 e4 }` | 1→C, 2→D, 3→E |
| `"CDE"` | Western | `\version "2.24.0" { c4 d4 e4 }` | Direct mapping |

### ✅ Invalid/Rejected Cases
| Input | Reason | Output | Notes |
|-------|--------|---------|-------|
| `"S R G"` | Has spaces | `\version "2.24.0" {  }` | Treated as plain text |
| `"S1G"` | Mixed systems | `\version "2.24.0" {  }` | Sargam + Number |
| `"SRC"` | Mixed systems | `\version "2.24.0" {  }` | Sargam + Western |
| `"xy"` | Invalid/short | Parse error | Not musical + too short |

### ✅ Backward Compatibility
| Input | Parsing Method | Output | Notes |
|-------|----------------|---------|-------|
| `"\|SRG"` | Grammar (barline) | `\version "2.24.0" { \| c4 d4 g4 }` | G→G due to grammar ambiguity |
| `"SRG"` | Consecutive | `\version "2.24.0" { c4 d4 e4 }` | G→E due to Sargam context |

## Architecture Benefits

### 1. Resolves Grammar Ambiguity
- Consecutive detection bypasses PEST ordered choice rules
- Ensures system-consistent interpretation of ambiguous characters
- 'G' correctly interpreted as Sargam Ga (N3) in Sargam context

### 2. Maintains Backward Compatibility
- Barline notation continues to work with existing grammar
- Complex notation still uses full PEST parsing capabilities
- No breaking changes to existing functionality

### 3. User Experience Improvements
- Enables natural input: `"SRG"` just works
- Clear distinction between consecutive and spaced input
- Proper error handling for invalid cases

## Technical Rationale

### Why Consecutive Detection First?
The original fallback approach failed because:
1. Grammar parsing of "SRmG" partially succeeded (created staves with mixed systems)
2. Fallback only triggered when `document.staves.is_empty()`
3. Mixed systems were never corrected

The priority approach ensures:
1. System-consistent detection happens before ambiguous grammar parsing
2. Clear separation between consecutive patterns and complex notation
3. Predictable behavior regardless of grammar parsing success/failure

### Why Minimum 3 Characters?
- Prevents false positives on short input like "G" or "am"
- Establishes clear musical intent (3+ notes form a musical phrase)
- Balances usability with precision

### Why Single-Line Only?
- Multi-line input likely requires more complex parsing (measures, staves, lyrics)
- Keeps consecutive detection focused and predictable
- Avoids interference with existing multi-stave parsing logic

## Future Considerations

### Potential Enhancements
1. **Accidental Support**: Handle "S#RbG" with sharps/flats in consecutive notation
2. **Octave Markers**: Support simple octave notation like "S'RG" (upper octave)
3. **Mixed Case Refinement**: More sophisticated system detection for edge cases

### Integration Points
- Web interface fully supports consecutive detection
- CLI commands work with both consecutive and traditional notation
- Pipeline processing maintains same interface for both parsing methods

## Conclusion

The consecutive notation implementation successfully resolves the core grammar ambiguity issue while maintaining full backward compatibility. The **SRGmPDN → 1234567** mapping now works correctly, with 'G' properly interpreted as Sargam Ga (N3 → Western E) rather than Western G (N5).

The solution provides a clean, predictable user experience where consecutive musical characters are automatically detected and parsed with system consistency, while preserving the full power of the PEST grammar for complex notation requiring barlines and advanced features.