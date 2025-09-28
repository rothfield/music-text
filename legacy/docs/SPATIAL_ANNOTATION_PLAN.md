# Spatial Annotation System Implementation Plan

## Overview
Implement underline-based spatial grouping for slurs (pre-lines) and beat groups (post-lines) using both tokens and attributes.

## Phase 1: Extend Data Model

### 1.1 Add Tokens to MusicalElement enum
```rust
pub enum MusicalElement {
    // Existing...
    Note(Note),
    Barline { source: Source },
    Space { count: usize, source: Source },
    
    // New spatial tokens
    SlurBegin { source: Source },
    SlurEnd { source: Source },
    BeatGroupBegin { source: Source },
    BeatGroupEnd { source: Source },
}
```

### 1.2 Add Attributes to All Elements
Extend Note, Barline, Space, and new token variants with:
```rust
pub in_slur: bool,
pub in_beat_group: bool,
```

## Phase 2: Enhanced Content Line Transformation

### 2.1 Integrated Spatial Logic
Modify `content_line.rs` transformer to:
1. Read text_lines_before for underline patterns (slurs)
2. Read text_lines_after for underline patterns (beat groups)
3. Transform content_line elements with spatial awareness
4. Insert tokens and set attributes during element creation

### 2.2 Underline Detection Algorithm
1. **Pre-lines**: Scan for `_` characters, track column spans for slurs
2. **Post-lines**: Scan for `_` characters, track column spans for beat groups
3. **Column mapping**: Map underline spans to content line element positions
4. **Boundary detection**: Insert Begin/End tokens at span boundaries
5. **Attribute setting**: Set in_slur/in_beat_group on elements within spans

## Phase 3: Implementation Details

### 3.1 Processing Flow
1. Parse stave structure normally (existing)
2. Enhanced content_line transformation:
   - Scan text lines for underlines
   - Process content elements with spatial context
   - Insert spatial tokens at boundaries
   - Set boolean attributes on all elements

### 3.2 Column Position Logic
- Map character positions in text lines to element positions in content line
- Handle spaces, barlines, and notes consistently
- Account for multi-character elements (e.g. accidentals)

## Phase 4: Testing & Validation

### 4.1 Test Cases
- Simple slur: `1___2` → SlurBegin, Note(1), Note(2), SlurEnd
- Overlapping annotations: pre-line slurs + post-line beat groups
- Complex patterns: multiple slurs, nested groups
- Edge cases: underlines at line boundaries

### 4.2 Integration Testing
- Verify tokens appear in correct positions
- Verify attributes are set correctly
- Test with existing rhythm FSM pipeline

## Key Design Decisions

1. **Dual Approach**: Both tokens (for precise boundaries) and attributes (for convenient queries)
2. **Column-Based**: Use character positions to map underlines to elements
3. **Single Pass**: Integrate spatial logic into existing content_line transformation
4. **No Separate Analysis**: Underline detection happens during tree transformation, not as separate step