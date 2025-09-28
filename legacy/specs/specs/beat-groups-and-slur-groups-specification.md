# Beat Groups and Slur Groups Specification

## Overview

This specification defines the spatial relationship rules for beat groups and slur groups in the music-text notation language. Both features use underscore symbols (`_`) but have different semantic meanings based on their spatial position relative to the content line.

## Symbol Disambiguation

### Spatial Context Determines Meaning

The same underscore symbol has different interpretations based on vertical position:

```
~   ~   ~     <- Upper line: ornaments, octave markers
_____         <- Upper line: slurs (musical phrasing)
|1 2 3 4|     <- Content line: musical notes
_____         <- Lower line: beat groups (rhythmic grouping)
.     .       <- Lower line: octave markers, syllables
```

### Symbol Definitions

- **Upper line underscores**: `_____` = **Slurs** (musical legato phrasing)
- **Lower line underscores**: `_____` = **Beat Groups** (rhythmic grouping indicators)

## Grammar Rules

### Beat Group Indicators

```ebnf
beat_group_indicator = "__" "_"*   // minimum 2 consecutive underscores
```

- **Minimum length**: 2 underscores (`__`)
- **Maximum length**: No limit
- **Position**: Lower line elements only
- **Semantic meaning**: Groups notes for rhythmic emphasis

### Slur Indicators

```ebnf
slur_indicator = "__" "_"*   // minimum 2 consecutive underscores
```

- **Minimum length**: 2 underscores (`__`)
- **Maximum length**: No limit
- **Position**: Upper line elements only
- **Semantic meaning**: Musical legato phrasing

## Processing Pipeline

### Phase 1: Consumption with Position Tracking

Both beat groups and slurs are consumed using move semantics with `PositionTracker`:

```rust
// Beat groups consumed from lower lines
let consumed_beat_groups = consume_beat_groups(lower_lines);

// Slurs consumed from upper lines
let consumed_slurs = consume_slurs(upper_lines);
```

**Consumption Process**:
1. Extract underscore sequences from source lines
2. Calculate precise start/end positions using `PositionTracker`
3. Move source values (consume) to prevent double-processing
4. Return positioned consumption data

### Phase 2: Direct Assignment

Exact spatial alignment between indicators and musical elements:

```rust
// Direct assignment for exact position matches
let (content_with_beat_groups, remaining_beat_groups) =
    assign_beat_groups_direct(content_elements, consumed_beat_groups);

let (content_with_slurs, remaining_slurs) =
    assign_slurs_direct(content_elements, consumed_slurs);
```

**Direct Assignment Rules**:
- Notes must fall within indicator span: `note_pos >= start_pos && note_pos <= end_pos`
- Minimum 2 musical elements required for group formation
- Overlap detection: existing assignments preserved with warnings
- Exact position matching only

### Phase 3: Fallback Assignment

Nearest available element assignment for unmatched indicators:

```rust
// Fallback assignment to nearest available elements
let (final_content, still_remaining_beat_groups) =
    assign_beat_groups_nearest(content_with_beat_groups, remaining_beat_groups);
```

**Fallback Assignment Rules**:
- Only assigns to elements without existing assignments
- Maximum distance: 5 columns from indicator position
- Prefers closer elements, then leftmost position
- Still requires minimum 2 elements for group formation

### Phase 4: Validation and Error Reporting

```rust
// Validate processing and report unprocessed indicators
let warnings = validate_beat_group_processing(&still_remaining_beat_groups);
```

## Assignment Rules

### Beat Group Assignment

**Minimum Requirements**:
- 2+ consecutive underscores in lower line
- 2+ musical elements (notes, rests, or dashes) within span
- Elements must not already belong to another beat group

**Assignment Process**:
1. Find all musical elements within underscore span
2. Check minimum element count (≥2)
3. Verify no existing beat group assignments (overlap detection)
4. Assign roles: Start → Middle → ... → End

**Supported Elements**:
- `Note`: Full beat group role assignment
- `Rest`: Counted for grouping, no role assignment
- `Dash`: Counted for grouping, no role assignment

### Slur Assignment

**Minimum Requirements**:
- 2+ consecutive underscores in upper line
- 2+ note elements within span (rests/dashes excluded)
- Notes must not already belong to another slur

**Assignment Process**:
1. Find all note elements within underscore span
2. Check minimum note count (≥2)
3. Verify no existing slur assignments
4. Assign roles: Start → Middle → ... → End

**Supported Elements**:
- `Note`: Full slur role assignment only
- `Rest`/`Dash`: Ignored for slur grouping

## Role Assignment

### Beat Group Roles

```rust
pub enum BeatGroupRole {
    Start,   // First element in beat group
    Middle,  // Intermediate elements
    End,     // Last element in beat group
}
```

### Slur Roles

```rust
pub enum SlurRole {
    Start,   // First note in slur phrase
    Middle,  // Intermediate notes
    End,     // Last note in slur phrase
}
```

### Role Assignment Logic

For both beat groups and slurs:

```rust
for (i, &element_index) in group_elements.iter().enumerate() {
    let role = if i == 0 {
        Role::Start
    } else if i == group_elements.len() - 1 {
        Role::End
    } else {
        Role::Middle
    };

    assign_role(element_index, role);
}
```

## Conflict Resolution

### Overlap Detection

When multiple indicators compete for the same musical elements:

1. **Existing assignments preserved**: No silent overwrites
2. **Warning messages generated**: Position-specific conflict reporting
3. **Fallback processing**: Conflicted indicators added to remaining pool
4. **Graceful degradation**: System continues processing other indicators

### Warning Examples

```
Warning: Beat group overlap detected at position 1:4 - existing assignment preserved
Warning: Slur overlap detected at position 1:6 - existing assignment preserved
```

### Unprocessed Indicators

Indicators that cannot be assigned generate validation warnings:

```
Unprocessed beat group indicator at position 3 (span: 3-5): 3 underscores could not be assigned to notes
Unprocessed slur indicator at position 7 (span: 7-9): 3 underscores could not be assigned to notes
```

## Distance Limits

### Fallback Assignment Distance

- **Maximum distance**: 5 columns from indicator position
- **Distance calculation**: Absolute difference between positions
- **Preference order**: Closer distance first, then leftmost position

```rust
let distance = if indicator_pos > note_pos {
    indicator_pos - note_pos
} else {
    note_pos - indicator_pos
};

if distance <= 5 {
    candidates.push((distance, note_pos, note_index));
}
```

## Implementation Examples

### Basic Beat Group

```
Input:
|1 2 3 4|
___   ___

Expected Output:
- Notes 1,2: beat_group = Start,End
- Notes 3,4: beat_group = Start,End
- All notes: in_beat_group = true
```

### Basic Slur

```
Input:
_____
|1 2 3 4|

Expected Output:
- Notes 1,2,3,4: slur = Start,Middle,Middle,End
- All notes: in_slur = true
```

### Overlap Conflict

```
Input:
_____
|1 2 3 4|
___   ___

Expected Output:
- Notes 1,2,3,4: slur = Start,Middle,Middle,End (upper line processed first)
- Beat group indicators: unprocessed (conflict detected)
- Warnings: overlap conflicts reported
```

### Fallback Assignment

```
Input:
|1  2    3|
  ___

Expected Output:
- Note 2: beat_group = Start (closest to indicator)
- Note 3: beat_group = End (within distance limit)
- Note 1: no assignment (too far from indicator)
```

## Error Handling

### Processing Errors

- **Invalid underscore count**: Less than 2 underscores ignored
- **Insufficient elements**: Groups with <2 elements rejected
- **Position conflicts**: Existing assignments preserved with warnings
- **Distance limits**: Elements beyond 5 columns ignored in fallback

### Validation Errors

- **Unconsumed indicators**: Source values not moved generate errors
- **Unprocessed groups**: Indicators that couldn't be assigned generate warnings
- **Position misalignment**: Spatial relationship validation failures

## Implementation Status

- ✅ **Beat group processing**: Full two-phase assignment with position tracking
- ✅ **Position tracking**: PositionTracker integration for precise alignment
- ✅ **Overlap detection**: Conflict detection with warning generation
- ✅ **Fallback assignment**: Nearest element assignment within distance limits
- ✅ **Validation**: Comprehensive error reporting for unprocessed indicators
- 🚧 **Slur processing**: Grammar defined, implementation pending
- ⚠️ **Cross-group conflicts**: Beat group vs slur priority rules need specification

## Future Enhancements

### Priority Rules

When both beat groups and slurs compete for the same notes:

1. **Processing order**: Upper line (slurs) before lower line (beat groups)
2. **Conflict resolution**: First assignment wins, subsequent assignments warned
3. **Hybrid assignment**: Allow notes to belong to both slur and beat group simultaneously

### Advanced Features

- **Nested groups**: Support for overlapping but non-conflicting groups
- **Group spanning**: Multi-measure group indicators
- **Dynamic fallback**: Intelligent element selection based on musical context

---

*This specification complements the main grammar specification and provides detailed implementation guidance for spatial relationship processing in the music-text notation language.*