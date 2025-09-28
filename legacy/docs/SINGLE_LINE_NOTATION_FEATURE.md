# Single-Line Notation Feature

**Date**: 2025-09-08  
**Feature**: Barline-free single-line musical notation  
**Status**: ✅ Implemented and Working  

## What We Built Today

We implemented automatic detection and parsing of **single-line musical notation without requiring barlines**. This allows users to input consecutive musical characters like `"SRG"`, `"123"`, or `"CDE"` and have them automatically recognized as valid musical notation.

## The Core Feature

### Input Transformation
```
Before: "SRG" → ERROR: "expected barline"
After:  "SRG" → \version "2.24.0" { c4 d4 e4 }
```

### Key Capability
**Automatic system detection**: The parser identifies which musical notation system (Number, Western, Sargam) is being used and applies consistent mapping throughout the input.

## How It Works

### 1. **Detection Criteria**
- ✅ **3+ consecutive characters**: Minimum threshold to establish musical intent
- ✅ **Single system consistency**: All characters must be from the same notation system  
- ✅ **Single line only**: Multi-line input uses traditional parsing
- ✅ **No spaces**: `"SRG"` = music, `"S R G"` = plain text

### 2. **Supported Systems**
| System | Characters | Example | Output |
|--------|------------|---------|---------|
| **Number** | `1234567` | `"123"` | `{ c4 d4 e4 }` |
| **Western** | `CDEFGAB` | `"CDE"` | `{ c4 d4 e4 }` |
| **Sargam** | `SRGMPDNsrgmpdn` | `"SRG"` | `{ c4 d4 e4 }` |

### 3. **System Mapping**
All notation systems map to the same Western output:
```
Numbers:  1 2 3 4 5 6 7  →  C D E F G A B
Western:  C D E F G A B  →  C D E F G A B  
Sargam:   S R G M P D N  →  C D E F G A B
```

## The Critical Fix: Sargam 'G' Mapping

### Problem We Solved
The most important fix was correcting the Sargam 'G' character mapping:

```
❌ WRONG: "SRmG" → { c4 d4 f4 g4 }  (G mapped as Western G = N5)
✅ FIXED: "SRmG" → { c4 d4 f4 e4 }  (G mapped as Sargam Ga = N3)
```

### Why This Mattered
- **User requirement**: `SRGmPDN → 1234567` (scale degree mapping)
- **Grammar ambiguity**: PEST parser was treating 'G' as Western before Sargam due to ordered choice rules
- **System consistency**: Mixed systems in single input created incorrect musical output

## Examples That Now Work

### ✅ **Valid Input**
```
"SRG"   → Sargam Sa-Re-Ga     → { c4 d4 e4 }
"SRmG"  → Sargam Sa-Re-Ma-Ga  → { c4 d4 f4 e4 }  ← Key fix
"123"   → Number 1-2-3         → { c4 d4 e4 }
"CDE"   → Western C-D-E        → { c4 d4 e4 }
```

### ❌ **Rejected Input** (treated as plain text)
```
"S R G" → Has spaces            → {  }  (empty)
"S1G"   → Mixed Sargam+Number  → {  }  (empty)
"SRC"   → Mixed Sargam+Western → {  }  (empty)
"xy"    → Invalid + too short  → Parse error
```

### ✅ **Backward Compatibility**
```
"|SRG"  → Traditional barline notation still works → { | c4 d4 g4 }
```
*(Note: G becomes Western G in traditional parsing due to grammar ambiguity)*

## Technical Implementation

### Architecture Decision: **Priority Parsing**
We changed the parsing strategy from "fallback" to "priority":

```rust
// NEW: Consecutive detection runs FIRST
pub fn parse_document(input: &str) -> Result<Document, String> {
    // Step 1: Check consecutive notation first
    if let Some(stave) = try_create_consecutive_stave(input) {
        return Ok(Document { staves: vec![stave], source });
    }
    
    // Step 2: Fall back to grammar parsing
    let document = build_document(input)?;
    Ok(document)
}
```

### Why Priority Over Fallback?
- **Fallback approach** only triggered when grammar parsing completely failed
- **Grammar parsing** of "SRmG" partially succeeded but created mixed notation systems
- **Priority approach** ensures system consistency before ambiguous grammar rules apply

### Web Interface Fix
Fixed trailing newline handling for web inputs:
```rust
let trimmed_input = input.trim();  // Handle "SRG\n" from web interface
```

## User Experience Impact

### 1. **Natural Input**
Users can now type musical patterns naturally:
- `SRG` for Sargam notation
- `123` for number notation  
- `CDE` for Western notation

### 2. **Clear Behavior**
- **Consecutive characters** = musical notation
- **Spaced characters** = plain text
- **Mixed systems** = rejected (plain text)

### 3. **Predictable Results**
- Same system throughout = musical output
- Different systems mixed = plain text output
- Complex notation = use traditional barline notation

## What This Enables

### For Users
- ✅ Quick musical sketching without barlines
- ✅ Natural notation system entry
- ✅ Immediate feedback on system consistency
- ✅ Clear distinction between music and text

### For Developers  
- ✅ Clean separation between simple and complex notation
- ✅ System-consistent parsing without grammar ambiguity
- ✅ Backward compatibility with existing barline notation
- ✅ Extensible foundation for future notation features

## The Bottom Line

**Single-line notation transforms this**:
```
❌ Before: "SRG" → ERROR: "expected barline"
✅ After:  "SRG" → Beautiful musical notation
```

This feature makes music-text more accessible by allowing natural, barline-free input for simple musical patterns while preserving the full power of the grammar system for complex notation. The critical Sargam mapping fix ensures that `SRGmPDN → 1234567` works correctly, making the system musically accurate and user-friendly.

**Result**: Users can now input consecutive musical characters naturally, and the system automatically detects the notation system and produces correct musical output with proper LilyPond rendering.