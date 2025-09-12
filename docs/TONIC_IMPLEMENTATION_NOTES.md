# Tonic Implementation Technical Notes

## Overview
Implemented a tonic-based movable-do system where `key: D` means "1 = D" (scale degree 1 is D), not "D major key signature".

## Architecture

### 1. Lexer Recognition
The lexer already recognized `key:` directives as metadata. No changes needed at lexer level.

### 2. Parser Injection (src/lib.rs)
```rust
// In unified_parser function
// Check for Key in metadata and inject Tonic item at the beginning
let key_str = metadata.attributes.get("Key")
    .or_else(|| metadata.attributes.get("key")); // Handle case sensitivity
if let Some(key_str) = key_str {
    if let Some(tonic_degree) = parse_key_to_degree(key_str) {
        elements.insert(0, rhythm_fsm::Item::Tonic(tonic_degree));
    }
}
```

### 3. FSM Item Enum (src/rhythm_fsm.rs)
Added new variant to Item enum:
```rust
pub enum Item {
    Beat(Beat),
    Barline(String),
    Breathmark,
    SlurStart,
    SlurEnd,
    Tonic(Degree),  // NEW: Carries the tonic degree
}
```

### 4. VexFlow Transposition (src/vexflow_js_generator.rs)
- Track `current_tonic` throughout processing
- Pass tonic to `degree_to_vexflow_key` function
- Use scale lookup tables for each tonic:
```rust
let scale_notes = match key_degree {
    N2 => ["d", "e", "f#", "g", "a", "b", "c#"], // D major scale
    // ... other scales
};
```

### 5. LilyPond Transposition (src/to_lilypond_src.rs)
- Added `transpose_degree` function for mathematical transposition
- Track `current_tonic` throughout processing
- Apply transposition before converting to LilyPond notation:
```rust
let transposed_degree = if let Some(tonic) = current_tonic {
    transpose_degree(degree, tonic)
} else {
    degree
};
```

## Key Design Decisions

### Tonic vs Key Signature
- **Tonic**: The reference pitch for scale degrees (movable-do)
- **Key Signature**: NOT automatically applied
- Always output in C major key signature with explicit accidentals
- This maintains the philosophy that tonic is about relative pitch, not Western key signatures

### Node Injection vs Metadata
- Tonic is injected as a node (`Item::Tonic`) at the beginning of the element stream
- This allows the FSM and converters to process it like any other musical element
- Maintains clean separation between parsing and conversion

### Case Sensitivity Fix
- Metadata stores "key" (lowercase) but some code looked for "Key" (capitalized)
- Fixed by checking both: `.get("Key").or_else(|| .get("key"))`

## Current Status
- ✅ VexFlow correctly outputs transposed notes (e.g., `d/4`, `e/4`, `fs/4` for key:D with input 1 2 3)
- ⚠️ LilyPond transposition partially working (mathematical transposition implemented but needs scale-based approach)
- 🔄 Web UI support pending
- 🔄 Comprehensive tests pending

## Testing
Test with:
```bash
echo "key: D\n1 2 3 4" > /tmp/test.123
NOTATION_OUTPUT_DIR="test_output" ./target/release/cli /tmp/test.123
```

Expected output:
- VexFlow: `["d/4"], ["e/4"], ["fs/4"], ["g/4"]`
- LilyPond: `d4 e4 fis4 g4` (with C major key signature)

## Future Work
1. Complete LilyPond transposition (use scale tables like VexFlow)
2. Add web UI support for key input
3. Test with all supported tonics (C, D, E, F, G, A, B, and flats)
4. Test with other notation systems (Western, Sargam)