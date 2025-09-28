# Spatial Assignment Specification

## Note on Formal Grammar Approach

A **2D array grammar** is the most precise name for a formal grammar that works with column/row alignment and rewriting sub-arrays into structured 2D outputs. This specification describes the implementation of such a 2D grammar for music-text notation, where spatial relationships between vertically aligned elements are formalized using production rules like:

```
[ content-line    ]
[ annotation-line ] → [ enhanced-content ]
```

See the main grammar specification (specs/grammar-specification.md) for the formal 2D grammar production rules that govern this spatial assignment system.

## Overview

This document specifies how musical annotations (octave markers, lyrics, slurs) are spatially assigned to notes in Music-Text notation using **move semantics**. The spatial assignment system processes multi-line input where annotations appear above and below the main musical content line, assigning them to notes based on their horizontal position with **explicit ownership transfer**.

## Architecture

### Move Pattern Principles

**Core Concept**: Data ownership is explicit and transfers through the pipeline. Once data is consumed/assigned, the source becomes `None`, preventing accidental reuse.

```rust
// Source ownership tracking
pub struct Source {
    pub value: Option<String>,  // None when moved/consumed
    pub position: Position,     // Always preserved for debugging
}
```

### Input Structure

Music-Text uses a **three-line annotation system** with tracked ownership:

```
.  :  .     <- Upper line (octave markers, slurs, chords)
1  2  3     <- Content line (notes, rests, barlines)
.  :  .     <- Lower line (octave markers, beat groupings)
```

## Data Structures with Move Semantics

### Annotation Elements

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpperElement {
    UpperOctaveMarker {
        marker: String,    // Owned marker value
        source: Source,    // Source becomes None when consumed
    },
    UpperUnderscores {
        value: String,     // Owned underscore sequence
        source: Source,    // Ownership tracking
    },
    Space {
        count: usize,      // Immutable space count
        source: Source,    // Position tracking only
    },
    // ... other variants
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LowerElement {
    LowerOctaveMarker {
        marker: String,    // Owned marker value
        source: Source,    // Source becomes None when consumed
    },
    LowerUnderscores {
        value: String,     // Owned underscore sequence
        source: Source,    // Ownership tracking
    },
    // ... other variants
}
```

### Assignment Tracker

```rust
pub struct SpatialAssigner {
    position: usize,
    consumed_markers: Vec<(usize, ConsumedMarker)>,
    consumed_slurs: Vec<(usize, ConsumedSlur)>,
    available_notes: Vec<(usize, &mut Note)>,
}

pub struct ConsumedMarker {
    pub octave_value: i8,
    pub original_source: Source,  // Source.value is now None
}

pub struct ConsumedSlur {
    pub start_pos: usize,
    pub end_pos: usize,
    pub original_source: Source,  // Source.value is now None
}
```

## Assignment Algorithms with Move Semantics

### 1. Octave Marker Assignment

#### Phase 1: Consumption and Collection

**Algorithm**: Extract markers from upper/lower lines, consuming ownership.

```rust
pub fn consume_octave_markers(mut upper_line: UpperLine, mut lower_line: LowerLine)
    -> (Vec<ConsumedMarker>, UpperLine, LowerLine) {

    let mut consumed_markers = Vec::new();
    let mut current_pos = 0;

    // Consume markers from upper line
    for element in &mut upper_line.elements {
        match element {
            UpperElement::UpperOctaveMarker { marker, source } => {
                let octave_value = octave_marker_to_number(&marker, true);

                // Move the marker value, mark source as consumed
                let consumed = ConsumedMarker {
                    octave_value,
                    original_source: Source {
                        value: source.value.take(), // Moves the value out
                        position: source.position.clone(),
                    },
                };

                consumed_markers.push((current_pos, consumed));
                // source.value is now None - marker is consumed
            }
            UpperElement::Space { count, .. } => current_pos += count,
            _ => current_pos += 1,
        }
    }

    // Similar process for lower line with negative octaves
    current_pos = 0;
    for element in &mut lower_line.elements {
        match element {
            LowerElement::LowerOctaveMarker { marker, source } => {
                let octave_value = octave_marker_to_number(&marker, false);

                let consumed = ConsumedMarker {
                    octave_value,
                    original_source: Source {
                        value: source.value.take(), // Moves the value out
                        position: source.position.clone(),
                    },
                };

                consumed_markers.push((current_pos, consumed));
            }
            LowerElement::Space { count, .. } => current_pos += count,
            _ => current_pos += 1,
        }
    }

    (consumed_markers, upper_line, lower_line)
}
```

#### Phase 2: Direct Assignment

**Algorithm**: Assign consumed markers to notes at exact positions.

```rust
pub fn assign_markers_direct(
    mut notes: Vec<Note>,
    mut consumed_markers: Vec<(usize, ConsumedMarker)>
) -> (Vec<Note>, Vec<(usize, ConsumedMarker)>) {

    let mut assigned_markers = Vec::new();
    let mut remaining_markers = Vec::new();

    // Extract note positions
    let note_positions: Vec<(usize, usize)> = notes.iter()
        .enumerate()
        .map(|(idx, note)| (note.pitch_string.source.position.column, idx))
        .collect();

    // Direct assignment pass
    for (marker_pos, consumed_marker) in consumed_markers {
        let mut assigned = false;

        for (note_pos, note_idx) in &note_positions {
            if marker_pos == *note_pos {
                // Transfer ownership of marker to note
                notes[*note_idx].octave = consumed_marker.octave_value;
                assigned_markers.push((marker_pos, consumed_marker));
                assigned = true;
                break;
            }
        }

        if !assigned {
            remaining_markers.push((marker_pos, consumed_marker));
        }
    }

    (notes, remaining_markers)
}
```

#### Phase 3: Nearest Neighbor Assignment

**Algorithm**: Assign remaining markers to closest unassigned notes.

```rust
pub fn assign_markers_nearest(
    mut notes: Vec<Note>,
    remaining_markers: Vec<(usize, ConsumedMarker)>
) -> Vec<Note> {

    let unassigned_notes: Vec<usize> = notes.iter()
        .enumerate()
        .filter(|(_, note)| note.octave == 0) // Default octave = unassigned
        .map(|(idx, _)| idx)
        .collect();

    for (marker_pos, consumed_marker) in remaining_markers {
        let mut best_distance = usize::MAX;
        let mut best_note_idx = None;

        for &note_idx in &unassigned_notes {
            let note_pos = notes[note_idx].pitch_string.source.position.column;
            let distance = if marker_pos > note_pos {
                marker_pos - note_pos
            } else {
                note_pos - marker_pos
            };

            if distance < best_distance {
                best_distance = distance;
                best_note_idx = Some(note_idx);
            }
        }

        // Transfer ownership to best match
        if let Some(note_idx) = best_note_idx {
            notes[note_idx].octave = consumed_marker.octave_value;
            // consumed_marker is now moved/consumed
        }
        // If no match found, marker is discarded (move semantics)
    }

    notes
}
```

### 2. Slur Assignment with Move Semantics

#### Algorithm: Consumption and Spatial Assignment

```rust
pub fn consume_and_assign_slurs(
    mut upper_line: UpperLine,
    mut notes: Vec<Note>
) -> (UpperLine, Vec<Note>) {

    let mut consumed_slurs = Vec::new();
    let mut current_pos = 0;

    // Consume slur segments from upper line
    for element in &mut upper_line.elements {
        match element {
            UpperElement::UpperUnderscores { value, source } => {
                if value.len() >= 2 { // Minimum 2 underscores for slur
                    let consumed = ConsumedSlur {
                        start_pos: current_pos,
                        end_pos: current_pos + value.len() - 1,
                        original_source: Source {
                            value: source.value.take(), // Move out the value
                            position: source.position.clone(),
                        },
                    };

                    consumed_slurs.push(consumed);
                    // source.value is now None - slur is consumed
                }
                current_pos += value.len();
            }
            UpperElement::Space { count, .. } => current_pos += count,
            _ => current_pos += 1,
        }
    }

    // Assign slur types to notes based on consumed slurs
    for consumed_slur in consumed_slurs {
        let note_positions: Vec<(usize, usize)> = notes.iter()
            .enumerate()
            .map(|(idx, note)| (note.pitch_string.source.position.column, idx))
            .collect();

        for (note_pos, note_idx) in note_positions {
            if note_pos >= consumed_slur.start_pos && note_pos <= consumed_slur.end_pos {
                notes[note_idx].in_slur = true;
                // First note in slur gets special marking for syllable assignment
                if note_pos == consumed_slur.start_pos {
                    // Mark as slur beginning for syllable assignment
                    notes[note_idx].slur_type = Some(SlurType::BeginSlur);
                } else {
                    notes[note_idx].slur_type = Some(SlurType::InSlur);
                }
            }
        }
        // consumed_slur is now moved/consumed
    }

    (upper_line, notes)
}
```

### 3. Syllable Assignment with Move Semantics

#### Algorithm: Syllable Consumption and Distribution

```rust
pub fn consume_and_assign_syllables(
    mut lyrics_lines: Vec<LyricsLine>,
    mut notes: Vec<Note>
) -> (Vec<LyricsLine>, Vec<Note>) {

    let mut consumed_syllables = Vec::new();

    // Consume all syllables from all lyrics lines
    for lyrics_line in &mut lyrics_lines {
        for syllable in &mut lyrics_line.syllables {
            // Move syllable content out
            if let Some(content) = syllable.source.value.take() {
                consumed_syllables.push(content);
                // syllable.source.value is now None - consumed
            }
        }
    }

    // Assign syllables to notes respecting slur boundaries
    let mut syllable_index = 0;
    for note in &mut notes {
        match note.slur_type {
            Some(SlurType::InSlur) => {
                // Notes in middle of slur don't get new syllables
                // No syllable assignment - respects melisma
            }
            Some(SlurType::BeginSlur) | None => {
                // First note of slur or unslurred notes get syllables
                if syllable_index < consumed_syllables.len() {
                    // Transfer ownership of syllable to note
                    note.syllable = Some(consumed_syllables[syllable_index].clone());
                    syllable_index += 1;
                }
            }
        }
    }

    // Remaining syllables are discarded (move semantics)
    (lyrics_lines, notes)
}
```

## Position Calculation with Move Semantics

### COLUMNAR Character Position System

**Critical Requirement**: Position calculation is **COLUMNAR**, with **move tracking** for consumed elements.

```rust
pub struct PositionTracker {
    current_pos: usize,
    consumed_positions: Vec<usize>, // Track consumed annotation positions
}

impl PositionTracker {
    pub fn advance_for_element(&mut self, element: &UpperElement) -> usize {
        let old_pos = self.current_pos;

        match element {
            UpperElement::Space { count, .. } => {
                self.current_pos += count;
            }
            UpperElement::UpperOctaveMarker { source, .. } => {
                // Only advance if not yet consumed
                if source.value.is_some() {
                    self.current_pos += 1;
                } else {
                    // Element was consumed - position already tracked
                    self.consumed_positions.push(old_pos);
                }
            }
            UpperElement::UpperUnderscores { value, source } => {
                if source.value.is_some() {
                    self.current_pos += value.len();
                } else {
                    // Slur was consumed
                    self.consumed_positions.push(old_pos);
                }
            }
            _ => self.current_pos += 1,
        }

        old_pos
    }
}
```

## Error Handling with Move Semantics

### Robustness Strategies

1. **Consumption Tracking**: Always verify if source data was consumed
2. **Orphaned Data**: Handle remaining unconsumed annotations gracefully
3. **Position Validation**: Ensure positions remain valid after moves

```rust
pub fn validate_consumption(upper_line: &UpperLine, lower_line: &LowerLine) -> Result<(), String> {
    // Check for unconsumed markers that should have been processed
    for element in &upper_line.elements {
        match element {
            UpperElement::UpperOctaveMarker { source, .. } => {
                if source.value.is_some() {
                    return Err(format!(
                        "Unconsumed octave marker at position {:?}",
                        source.position
                    ));
                }
            }
            UpperElement::UpperUnderscores { source, .. } => {
                if source.value.is_some() {
                    return Err(format!(
                        "Unconsumed slur at position {:?}",
                        source.position
                    ));
                }
            }
            _ => {} // Spaces and other elements don't need consumption
        }
    }

    // Similar validation for lower line
    Ok(())
}
```

### Edge Cases with Move Semantics

#### More Markers Than Notes
```rust
pub fn handle_excess_markers(excess_markers: Vec<ConsumedMarker>) -> Vec<String> {
    // Collect warnings about unused markers
    excess_markers.into_iter()
        .map(|marker| format!(
            "Unused octave marker '{}' at position {:?}",
            marker.octave_value,
            marker.original_source.position
        ))
        .collect()
}
```

#### Partial Consumption
```rust
pub fn handle_partial_slur_consumption(
    partially_consumed_slurs: Vec<ConsumedSlur>
) -> Result<(), String> {
    for slur in partially_consumed_slurs {
        if slur.end_pos - slur.start_pos < 1 {
            return Err(format!(
                "Invalid slur length at position {:?}",
                slur.original_source.position
            ));
        }
    }
    Ok(())
}
```

## Examples with Move Semantics

### Complete Spatial Assignment Example

Input:
```
.  :  .     [C7]
S  R  G  M  P
_     :     _
ga ma dha ni sa
```

Processing with Move Semantics:

1. **Marker Consumption**:
   ```rust
   // Before consumption
   UpperElement::UpperOctaveMarker {
       marker: ".".to_string(),
       source: Source {
           value: Some(".".to_string()),
           position: Position { line: 0, column: 0 }
       }
   }

   // After consumption
   UpperElement::UpperOctaveMarker {
       marker: ".".to_string(),
       source: Source {
           value: None,  // Consumed!
           position: Position { line: 0, column: 0 }
       }
   }
   ```

2. **Assignment Results**:
   - S: +1 (from consumed `.` at column 0)
   - R: +2 (from consumed `:` at column 3)
   - G: +1 (from consumed `.` at column 6)
   - M: 0 (no marker available)
   - P: 0 (no marker available)

3. **Syllable Transfer**:
   ```rust
   // Syllables consumed from lyrics lines
   consumed_syllables: ["ga", "ma", "dha", "ni", "sa"]

   // Transferred to notes
   notes[0].syllable = Some("ga".to_string())  // S
   notes[1].syllable = Some("ma".to_string())  // R
   notes[2].syllable = Some("dha".to_string()) // G
   notes[3].syllable = Some("ni".to_string())  // M
   notes[4].syllable = Some("sa".to_string())  // P
   ```

### Slur with Move Semantics Example

Input:
```
    ___            <- Upper line slur
1   2 3 4          <- Content line notes
hel lo  world      <- Lyrics line
```

Processing:
1. **Slur Consumption**: Upper line underscores consumed, positions 4-6 marked as slur
2. **Syllable Assignment with Move**:
   ```rust
   // Before processing
   lyrics_syllables: [
       Syllable { content: "hel", source: Source { value: Some("hel"), ... } },
       Syllable { content: "lo", source: Source { value: Some("lo"), ... } },
       Syllable { content: "world", source: Source { value: Some("world"), ... } }
   ]

   // After consumption and assignment
   notes[0].syllable = Some("hel")    // Note 1: BeginSlur - gets syllable
   notes[1].syllable = None           // Note 2: InSlur - melisma
   notes[2].syllable = None           // Note 3: InSlur - melisma
   notes[3].syllable = Some("lo")     // Note 4: no slur - gets syllable

   // "world" remains for future notes
   remaining_syllables: ["world"]

   // Lyrics line sources now consumed
   lyrics_syllables: [
       Syllable { content: "hel", source: Source { value: None, ... } },
       Syllable { content: "lo", source: Source { value: None, ... } },
       Syllable { content: "world", source: Source { value: None, ... } }
   ]
   ```

## Implementation Pipeline

### Data Flow with Move Semantics

```
Input Text → Parser → AST with Sources
    ↓
Spatial Assigner → Consume Annotations → Assign to Notes
    ↓
Validation → Check Consumption → Generate Warnings
    ↓
Enhanced AST → Rhythm Analysis → Rendering
```

### Architecture Integration

```rust
// Pipeline function signature
pub fn process_spatial_assignments(
    mut document: Document
) -> Result<(Document, Vec<String>), String> {

    let mut warnings = Vec::new();

    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            let (enhanced_stave, stave_warnings) = process_stave_spatial(stave)?;
            *stave = enhanced_stave;
            warnings.extend(stave_warnings);
        }
    }

    Ok((document, warnings))
}

fn process_stave_spatial(
    mut stave: Stave
) -> Result<(Stave, Vec<String>), String> {

    // Extract lines for processing
    let (upper_lines, content_line, lower_lines, lyrics_lines) =
        extract_lines_for_spatial(&mut stave)?;

    // Process with move semantics
    let (notes, warnings) = assign_spatial_annotations(
        upper_lines, content_line, lower_lines, lyrics_lines
    )?;

    // Update stave with processed content
    update_stave_with_processed_content(&mut stave, notes)?;

    Ok((stave, warnings))
}
```

## Performance Considerations

### Move Optimization

- **Zero-copy Transfers**: Annotations moved directly to notes without cloning
- **Lazy Evaluation**: Only process annotations when assignment is needed
- **Memory Efficiency**: Consumed sources become `None`, freeing memory early

### Algorithmic Complexity with Moves

- **Marker Assignment**: O(M + N) where M = markers, N = notes (single pass)
- **Slur Assignment**: O(S + N) where S = slur segments, N = notes
- **Memory**: O(1) additional overhead for move tracking

## Testing Requirements

### Move Semantics Validation

1. **Consumption Tests**: Verify all annotations are properly consumed
2. **Orphan Detection**: Test handling of unconsumed annotations
3. **Memory Tests**: Ensure no memory leaks from incomplete moves
4. **Position Preservation**: Verify positions remain valid after consumption

```rust
#[test]
fn test_octave_marker_consumption() {
    let input = ". : .\n1 2 3";
    let mut document = parse_document(input).unwrap();
    let (processed, warnings) = process_spatial_assignments(document).unwrap();

    // Verify all markers were consumed
    for element in &processed.elements {
        if let DocumentElement::Stave(stave) = element {
            for line in &stave.lines {
                if let StaveLine::Upper(upper_line) = line {
                    for element in &upper_line.elements {
                        if let UpperElement::UpperOctaveMarker { source, .. } = element {
                            assert!(source.value.is_none(), "Marker should be consumed");
                        }
                    }
                }
            }
        }
    }

    assert!(warnings.is_empty(), "Should have no warnings for perfect assignment");
}
```

## Future Extensions

### Advanced Move Patterns

1. **Conditional Moves**: Move annotations only when certain conditions are met
2. **Rollback Capability**: Restore consumed annotations if assignment fails
3. **Batch Moves**: Process multiple staves with shared annotation pools
4. **Cross-stave Moves**: Transfer annotations between related staves

```rust
// Future: Conditional assignment with rollback
pub struct ConditionalAssigner {
    checkpoints: Vec<AssignmentCheckpoint>,
}

pub struct AssignmentCheckpoint {
    consumed_markers: Vec<ConsumedMarker>,
    assignment_state: AssignmentState,
}

impl ConditionalAssigner {
    pub fn try_assign_with_rollback(&mut self, condition: AssignmentCondition)
        -> Result<(), AssignmentError> {

        let checkpoint = self.create_checkpoint();

        match self.attempt_assignment(condition) {
            Ok(result) => Ok(result),
            Err(err) => {
                self.rollback_to_checkpoint(checkpoint);
                Err(err)
            }
        }
    }
}
```

This specification now reflects the move pattern where data ownership is explicit, consumption is tracked, and the system prevents accidental reuse of processed annotations while maintaining complete traceability for debugging purposes.