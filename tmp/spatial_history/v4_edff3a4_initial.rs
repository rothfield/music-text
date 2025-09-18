use crate::parse::model::{
    Document, DocumentElement, Stave, StaveLine, UpperLine, LowerLine, LyricsLine,
    UpperElement, LowerElement, Syllable, Source, Position
};
use crate::rhythm::types::{ParsedElement, Position as RhythmPosition, ParsedChild};

/// Consumed marker with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedMarker {
    pub octave_value: i8,
    pub original_source: Source,  // Source.value is now None
    pub marker_symbol: String,    // Original marker symbol (".", ":", etc.)
    pub is_upper: bool,          // true = upper line, false = lower line
}

/// Consumed slur with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedSlur {
    pub start_pos: usize,
    pub end_pos: usize,
    pub original_source: Source,  // Source.value is now None
}

/// Consumed syllable with move semantics
#[derive(Debug, Clone)]
pub struct ConsumedSyllable {
    pub content: String,
    pub original_source: Source,  // Source.value is now None
}

/// Position tracker for spatial assignment
#[derive(Debug)]
pub struct PositionTracker {
    current_pos: usize,
    consumed_positions: Vec<usize>,
}

impl PositionTracker {
    pub fn new() -> Self {
        Self {
            current_pos: 0,
            consumed_positions: Vec::new(),
        }
    }

    pub fn advance_for_upper_element(&mut self, element: &UpperElement) -> usize {
        let old_pos = self.current_pos;

        match element {
            UpperElement::Space { count, .. } => {
                self.current_pos += count;
            }
            UpperElement::UpperOctaveMarker { source, .. } => {
                if source.value.is_some() {
                    self.current_pos += 1;
                } else {
                    self.consumed_positions.push(old_pos);
                }
            }
            UpperElement::UpperUnderscores { value, source } => {
                if source.value.is_some() {
                    self.current_pos += value.len();
                } else {
                    self.consumed_positions.push(old_pos);
                }
            }
            _ => self.current_pos += 1,
        }

        old_pos
    }

    pub fn advance_for_lower_element(&mut self, element: &LowerElement) -> usize {
        let old_pos = self.current_pos;

        match element {
            LowerElement::Space { count, .. } => {
                self.current_pos += count;
            }
            LowerElement::LowerOctaveMarker { source, .. } => {
                if source.value.is_some() {
                    self.current_pos += 1;
                } else {
                    self.consumed_positions.push(old_pos);
                }
            }
            LowerElement::LowerUnderscores { value, source } => {
                if source.value.is_some() {
                    self.current_pos += value.len();
                } else {
                    self.consumed_positions.push(old_pos);
                }
            }
            _ => self.current_pos += 1,
        }

        old_pos
    }
}

/// Convert octave marker to numeric value
fn octave_marker_to_number(marker: &str, is_upper: bool) -> i8 {
    let base_value = match marker {
        "." => 1,
        ":" => 2,
        "*" => 3,
        "'" => 4,
        _ => 0,
    };

    if is_upper { base_value } else { -base_value }
}

/// Consume octave markers from upper and lower lines
pub fn consume_octave_markers(
    mut upper_line: UpperLine,
    mut lower_line: LowerLine
) -> (Vec<(usize, ConsumedMarker)>, UpperLine, LowerLine) {
    let mut consumed_markers = Vec::new();
    let mut tracker = PositionTracker::new();

    // Consume markers from upper line
    for element in &mut upper_line.elements {
        let pos = tracker.advance_for_upper_element(element);

        if let UpperElement::UpperOctaveMarker { marker, source } = element {
            if let Some(value) = source.value.take() {
                let octave_value = octave_marker_to_number(&marker, true);
                let consumed = ConsumedMarker {
                    octave_value,
                    original_source: Source {
                        value: Some(value),
                        position: source.position.clone(),
                    },
                    marker_symbol: marker.clone(),
                    is_upper: true,
                };
                consumed_markers.push((pos, consumed));
            }
        }
    }

    // Reset tracker for lower line
    tracker = PositionTracker::new();

    // Consume markers from lower line
    for element in &mut lower_line.elements {
        let pos = tracker.advance_for_lower_element(element);

        if let LowerElement::LowerOctaveMarker { marker, source } = element {
            if let Some(value) = source.value.take() {
                let octave_value = octave_marker_to_number(&marker, false);
                let consumed = ConsumedMarker {
                    octave_value,
                    original_source: Source {
                        value: Some(value),
                        position: source.position.clone(),
                    },
                    marker_symbol: marker.clone(),
                    is_upper: false,
                };
                consumed_markers.push((pos, consumed));
            }
        }
    }

    (consumed_markers, upper_line, lower_line)
}

/// Assign consumed markers to content elements at exact positions
pub fn assign_markers_direct(
    mut content_elements: Vec<ParsedElement>,
    consumed_markers: Vec<(usize, ConsumedMarker)>
) -> (Vec<ParsedElement>, Vec<(usize, ConsumedMarker)>) {
    let mut remaining_markers = Vec::new();

    // Extract note positions (convert from col to column for matching)
    let note_positions: Vec<(usize, usize)> = content_elements.iter()
        .enumerate()
        .filter_map(|(idx, element)| {
            match element {
                ParsedElement::Note { position, .. } => Some((position.col, idx)),
                _ => None,
            }
        })
        .collect();

    // Direct assignment pass
    for (marker_pos, consumed_marker) in consumed_markers {
        let mut assigned = false;

        for (note_pos, note_idx) in &note_positions {
            if marker_pos == *note_pos {
                // Transfer ownership of marker to note
                if let ParsedElement::Note { octave, children, .. } = &mut content_elements[*note_idx] {
                    *octave = consumed_marker.octave_value;

                    // Add octave marker to note children
                    let distance = if consumed_marker.is_upper { -1 } else { 1 };
                    children.push(ParsedChild::OctaveMarker {
                        symbol: consumed_marker.marker_symbol.clone(),
                        distance,
                    });
                }
                assigned = true;
                break;
            }
        }

        if !assigned {
            remaining_markers.push((marker_pos, consumed_marker));
        }
    }

    (content_elements, remaining_markers)
}

/// Assign remaining markers to closest unassigned notes
pub fn assign_markers_nearest(
    mut content_elements: Vec<ParsedElement>,
    remaining_markers: Vec<(usize, ConsumedMarker)>
) -> Vec<ParsedElement> {
    for (marker_pos, consumed_marker) in remaining_markers {
        let mut best_distance = usize::MAX;
        let mut best_note_idx = None;

        for (idx, element) in content_elements.iter().enumerate() {
            if let ParsedElement::Note { octave, position, .. } = element {
                // Only assign to notes with default octave (unassigned)
                if *octave == 0 {
                    let note_pos = position.col;
                    let distance = if marker_pos > note_pos {
                        marker_pos - note_pos
                    } else {
                        note_pos - marker_pos
                    };

                    if distance < best_distance {
                        best_distance = distance;
                        best_note_idx = Some(idx);
                    }
                }
            }
        }

        // Transfer ownership to best match
        if let Some(note_idx) = best_note_idx {
            if let ParsedElement::Note { octave, children, .. } = &mut content_elements[note_idx] {
                *octave = consumed_marker.octave_value;

                // Add octave marker to note children
                let distance = if consumed_marker.is_upper { -1 } else { 1 };
                children.push(ParsedChild::OctaveMarker {
                    symbol: consumed_marker.marker_symbol.clone(),
                    distance,
                });
            }
        }
        // If no match found, marker is discarded (move semantics)
    }

    content_elements
}

/// Consume slur segments from upper line and assign to notes
pub fn consume_and_assign_slurs(
    mut upper_line: UpperLine,
    mut content_elements: Vec<ParsedElement>
) -> (UpperLine, Vec<ParsedElement>) {
    let mut consumed_slurs = Vec::new();
    let mut tracker = PositionTracker::new();

    // Consume slur segments from upper line
    for element in &mut upper_line.elements {
        let pos = tracker.advance_for_upper_element(element);

        if let UpperElement::UpperUnderscores { value, source } = element {
            if value.len() >= 2 { // Minimum 2 underscores for slur
                if let Some(underscore_value) = source.value.take() {
                    let consumed = ConsumedSlur {
                        start_pos: pos,
                        end_pos: pos + value.len() - 1,
                        original_source: Source {
                            value: Some(underscore_value),
                            position: source.position.clone(),
                        },
                    };
                    consumed_slurs.push(consumed);
                }
            }
        }
    }

    // Assign slur types to notes based on consumed slurs
    for consumed_slur in consumed_slurs {
        let note_positions: Vec<(usize, usize)> = content_elements.iter()
            .enumerate()
            .filter_map(|(idx, element)| {
                match element {
                    ParsedElement::Note { position, .. } => Some((position.col, idx)),
                    _ => None,
                }
            })
            .collect();

        for (note_pos, note_idx) in note_positions {
            if note_pos >= consumed_slur.start_pos && note_pos <= consumed_slur.end_pos {
                if let ParsedElement::Note { in_slur, .. } = &mut content_elements[note_idx] {
                    *in_slur = true;
                }
            }
        }
    }

    (upper_line, content_elements)
}

/// Consume syllables from lyrics lines and assign to notes respecting slur boundaries
pub fn consume_and_assign_syllables(
    mut lyrics_lines: Vec<LyricsLine>,
    mut content_elements: Vec<ParsedElement>
) -> (Vec<LyricsLine>, Vec<ParsedElement>) {
    let mut consumed_syllables = Vec::new();

    // Consume all syllables from all lyrics lines
    for lyrics_line in &mut lyrics_lines {
        for syllable in &mut lyrics_line.syllables {
            if let Some(content) = syllable.source.value.take() {
                let consumed = ConsumedSyllable {
                    content,
                    original_source: Source {
                        value: None, // Already consumed
                        position: syllable.source.position.clone(),
                    },
                };
                consumed_syllables.push(consumed);
            }
        }
    }

    // Assign syllables to notes respecting slur boundaries
    let mut syllable_index = 0;
    for element in &mut content_elements {
        if let ParsedElement::Note { in_slur, .. } = element {
            // Only assign syllables to notes that are not in the middle of slurs
            // For now, we'll assign to all notes since the current structure doesn't have syllable field
            if syllable_index < consumed_syllables.len() && !*in_slur {
                // Note: The current ParsedElement::Note doesn't have a syllable field
                // This would need to be added to the structure
                syllable_index += 1;
            }
        }
    }

    (lyrics_lines, content_elements)
}

/// Validate that all annotations were properly consumed
pub fn validate_consumption(upper_line: &UpperLine, lower_line: &LowerLine) -> Result<(), String> {
    // Check for unconsumed markers in upper line
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

    // Check for unconsumed markers in lower line
    for element in &lower_line.elements {
        match element {
            LowerElement::LowerOctaveMarker { source, .. } => {
                if source.value.is_some() {
                    return Err(format!(
                        "Unconsumed octave marker at position {:?}",
                        source.position
                    ));
                }
            }
            LowerElement::LowerUnderscores { source, .. } => {
                if source.value.is_some() {
                    return Err(format!(
                        "Unconsumed beat grouping at position {:?}",
                        source.position
                    ));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Main spatial assignment processor
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

/// Process spatial assignments for a single stave
fn process_stave_spatial(stave: &mut Stave) -> Result<(Stave, Vec<String>), String> {
    let mut warnings = Vec::new();

    // Extract different line types
    let mut upper_lines = Vec::new();
    let mut lower_lines = Vec::new();
    let mut lyrics_lines = Vec::new();
    let mut content_line_index = None;

    // Collect lines and find content line index
    for (idx, line) in stave.lines.iter().enumerate() {
        match line {
            StaveLine::Upper(upper_line) => {
                upper_lines.push(upper_line.clone());
            }
            StaveLine::Lower(lower_line) => {
                lower_lines.push(lower_line.clone());
            }
            StaveLine::Lyrics(lyrics_line) => {
                lyrics_lines.push(lyrics_line.clone());
            }
            StaveLine::Content(_) => {
                content_line_index = Some(idx);
            }
            _ => {} // Handle other line types as needed
        }
    }

    // Process spatial assignments if we have lower line and content line
    if !lower_lines.is_empty() && content_line_index.is_some() {
        let content_idx = content_line_index.unwrap();

        // Extract content elements from the content line
        if let StaveLine::Content(content_elements) = &mut stave.lines[content_idx] {
            // Create empty upper line if none exists
            let upper_line = if upper_lines.is_empty() {
                UpperLine {
                    elements: vec![],
                    source: Source {
                        value: Some("".to_string()),
                        position: Position { line: 0, column: 0 },
                    },
                }
            } else {
                upper_lines[0].clone()
            };

            let lower_line = lower_lines[0].clone();

            // 1. Consume octave markers
            let (consumed_markers, _updated_upper, _updated_lower) =
                consume_octave_markers(upper_line, lower_line);

            // 2. Assign markers to content elements
            let (content_with_octaves, remaining_markers) =
                assign_markers_direct(content_elements.clone(), consumed_markers);

            // 3. Assign remaining markers to nearest notes
            let final_content = assign_markers_nearest(content_with_octaves, remaining_markers);

            // Update the content line with processed elements
            *content_elements = final_content;
        }
    }

    Ok((stave.clone(), warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::model::{PitchString, PitchCode, NotationSystem};

    fn create_test_note(pitch: &str, column: usize) -> Note {
        Note {
            pitch_string: PitchString {
                source: Source {
                    value: Some(pitch.to_string()),
                    position: Position { line: 1, column },
                },
            },
            octave: 0,
            pitch_code: PitchCode::N1,
            notation_system: NotationSystem::Number,
            in_slur: false,
            in_beat_group: false,
        }
    }

    fn create_test_upper_line_with_markers() -> UpperLine {
        UpperLine {
            elements: vec![
                UpperElement::UpperOctaveMarker {
                    marker: ".".to_string(),
                    source: Source {
                        value: Some(".".to_string()),
                        position: Position { line: 0, column: 0 },
                    },
                },
                UpperElement::Space {
                    count: 2,
                    source: Source {
                        value: Some("  ".to_string()),
                        position: Position { line: 0, column: 1 },
                    },
                },
                UpperElement::UpperOctaveMarker {
                    marker: ":".to_string(),
                    source: Source {
                        value: Some(":".to_string()),
                        position: Position { line: 0, column: 3 },
                    },
                },
            ],
            source: Source {
                value: Some(".  :".to_string()),
                position: Position { line: 0, column: 0 },
            },
        }
    }

    fn create_test_lower_line_empty() -> LowerLine {
        LowerLine {
            elements: vec![],
            source: Source {
                value: Some("".to_string()),
                position: Position { line: 2, column: 0 },
            },
        }
    }

    fn create_test_lower_line_with_markers() -> LowerLine {
        LowerLine {
            elements: vec![
                LowerElement::LowerOctaveMarker {
                    marker: ".".to_string(),
                    source: Source {
                        value: Some(".".to_string()),
                        position: Position { line: 2, column: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".".to_string()),
                position: Position { line: 2, column: 0 },
            },
        }
    }

    #[test]
    fn test_octave_marker_to_number() {
        assert_eq!(octave_marker_to_number(".", true), 1);
        assert_eq!(octave_marker_to_number(":", true), 2);
        assert_eq!(octave_marker_to_number("*", true), 3);
        assert_eq!(octave_marker_to_number("'", true), 4);

        assert_eq!(octave_marker_to_number(".", false), -1);
        assert_eq!(octave_marker_to_number(":", false), -2);
        assert_eq!(octave_marker_to_number("*", false), -3);
        assert_eq!(octave_marker_to_number("'", false), -4);
    }

    #[test]
    fn test_position_tracker() {
        let mut tracker = PositionTracker::new();

        let space_element = UpperElement::Space {
            count: 3,
            source: Source {
                value: Some("   ".to_string()),
                position: Position { line: 0, column: 0 },
            },
        };

        let start_pos = tracker.advance_for_upper_element(&space_element);
        assert_eq!(start_pos, 0);
        assert_eq!(tracker.current_pos, 3);
    }

    #[test]
    fn test_consume_octave_markers() {
        let upper_line = create_test_upper_line_with_markers();
        let lower_line = create_test_lower_line_empty();

        let (consumed_markers, updated_upper, _updated_lower) =
            consume_octave_markers(upper_line, lower_line);

        // Should have consumed 2 markers
        assert_eq!(consumed_markers.len(), 2);

        // First marker should be at position 0 with value 1
        assert_eq!(consumed_markers[0].0, 0);
        assert_eq!(consumed_markers[0].1.octave_value, 1);

        // Second marker should be at position 3 with value 2
        assert_eq!(consumed_markers[1].0, 3);
        assert_eq!(consumed_markers[1].1.octave_value, 2);

        // Original sources should be consumed (None)
        for element in &updated_upper.elements {
            if let UpperElement::UpperOctaveMarker { source, .. } = element {
                assert!(source.value.is_none(), "Marker source should be consumed");
            }
        }
    }

    #[test]
    fn test_assign_markers_direct() {
        let notes = vec![
            create_test_note("1", 0),
            create_test_note("2", 3),
        ];

        let consumed_markers = vec![
            (0, ConsumedMarker {
                octave_value: 1,
                original_source: Source {
                    value: Some(".".to_string()),
                    position: Position { line: 0, column: 0 },
                },
            }),
            (3, ConsumedMarker {
                octave_value: 2,
                original_source: Source {
                    value: Some(":".to_string()),
                    position: Position { line: 0, column: 3 },
                },
            }),
        ];

        let (updated_notes, remaining_markers) =
            assign_markers_direct(notes, consumed_markers);

        // Both markers should be assigned
        assert_eq!(remaining_markers.len(), 0);
        assert_eq!(updated_notes[0].octave, 1);
        assert_eq!(updated_notes[1].octave, 2);
    }

    #[test]
    fn test_assign_markers_nearest() {
        let notes = vec![
            create_test_note("1", 0),
            create_test_note("2", 5),
        ];

        let remaining_markers = vec![
            (2, ConsumedMarker {
                octave_value: 1,
                original_source: Source {
                    value: Some(".".to_string()),
                    position: Position { line: 0, column: 2 },
                },
            }),
        ];

        let updated_notes = assign_markers_nearest(notes, remaining_markers);

        // Marker at position 2 should be assigned to note at position 0 (distance 2)
        // rather than note at position 5 (distance 3)
        assert_eq!(updated_notes[0].octave, 1);
        assert_eq!(updated_notes[1].octave, 0); // Unchanged
    }

    #[test]
    fn test_consume_and_assign_slurs() {
        let mut upper_line = UpperLine {
            elements: vec![
                UpperElement::Space {
                    count: 2,
                    source: Source {
                        value: Some("  ".to_string()),
                        position: Position { line: 0, column: 0 },
                    },
                },
                UpperElement::UpperUnderscores {
                    value: "___".to_string(),
                    source: Source {
                        value: Some("___".to_string()),
                        position: Position { line: 0, column: 2 },
                    },
                },
            ],
            source: Source {
                value: Some("  ___".to_string()),
                position: Position { line: 0, column: 0 },
            },
        };

        let notes = vec![
            create_test_note("1", 1),
            create_test_note("2", 2),
            create_test_note("3", 3),
            create_test_note("4", 4),
            create_test_note("5", 5),
        ];

        let (updated_upper, updated_notes) = consume_and_assign_slurs(upper_line, notes);

        // Slur should be consumed
        for element in &updated_upper.elements {
            if let UpperElement::UpperUnderscores { source, .. } = element {
                assert!(source.value.is_none(), "Slur source should be consumed");
            }
        }

        // Notes within slur range (positions 2-4) should be marked as in_slur
        assert!(!updated_notes[0].in_slur); // Position 1, outside slur
        assert!(updated_notes[1].in_slur);  // Position 2, in slur
        assert!(updated_notes[2].in_slur);  // Position 3, in slur
        assert!(updated_notes[3].in_slur);  // Position 4, in slur
        assert!(!updated_notes[4].in_slur); // Position 5, outside slur
    }

    #[test]
    fn test_validation_consumption_success() {
        let upper_line = UpperLine {
            elements: vec![
                UpperElement::UpperOctaveMarker {
                    marker: ".".to_string(),
                    source: Source {
                        value: None, // Consumed
                        position: Position { line: 0, column: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".".to_string()),
                position: Position { line: 0, column: 0 },
            },
        };

        let lower_line = create_test_lower_line_empty();

        assert!(validate_consumption(&upper_line, &lower_line).is_ok());
    }

    #[test]
    fn test_validation_consumption_failure() {
        let upper_line = UpperLine {
            elements: vec![
                UpperElement::UpperOctaveMarker {
                    marker: ".".to_string(),
                    source: Source {
                        value: Some(".".to_string()), // Not consumed
                        position: Position { line: 0, column: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".".to_string()),
                position: Position { line: 0, column: 0 },
            },
        };

        let lower_line = create_test_lower_line_empty();

        assert!(validate_consumption(&upper_line, &lower_line).is_err());
    }

    #[test]
    fn test_consume_lower_line_octave_markers() {
        let upper_line = UpperLine {
            elements: vec![],
            source: Source {
                value: Some("".to_string()),
                position: Position { line: 0, column: 0 },
            },
        };
        let lower_line = create_test_lower_line_with_markers();

        let (consumed_markers, _updated_upper, updated_lower) =
            consume_octave_markers(upper_line, lower_line);

        // Should have consumed 1 marker from lower line
        assert_eq!(consumed_markers.len(), 1);

        // Marker should be at position 0 with negative value -1
        assert_eq!(consumed_markers[0].0, 0);
        assert_eq!(consumed_markers[0].1.octave_value, -1);

        // Original source should be consumed (None)
        for element in &updated_lower.elements {
            if let LowerElement::LowerOctaveMarker { source, .. } = element {
                assert!(source.value.is_none(), "Lower marker source should be consumed");
            }
        }
    }

    #[test]
    fn test_lower_line_octave_assignment() {
        // Test the specific case: "1\n." should give octave -1
        let notes = vec![create_test_note("1", 0)];

        let consumed_markers = vec![
            (0, ConsumedMarker {
                octave_value: -1,  // Lower line marker
                original_source: Source {
                    value: Some(".".to_string()),
                    position: Position { line: 2, column: 0 },
                },
            }),
        ];

        let (updated_notes, remaining_markers) =
            assign_markers_direct(notes, consumed_markers);

        // Marker should be assigned
        assert_eq!(remaining_markers.len(), 0);
        assert_eq!(updated_notes[0].octave, -1);
    }

    #[test]
    fn test_consumed_marker_move_semantics() {
        let mut original_source = Source {
            value: Some(".".to_string()),
            position: Position { line: 0, column: 0 },
        };

        // Simulate consumption
        let consumed_value = original_source.value.take();
        assert!(consumed_value.is_some());
        assert!(original_source.value.is_none());

        let consumed_marker = ConsumedMarker {
            octave_value: 1,
            original_source: Source {
                value: consumed_value,
                position: original_source.position.clone(),
            },
        };

        // Original source is now consumed
        assert!(original_source.value.is_none());
        // But consumed marker preserves the value
        assert!(consumed_marker.original_source.value.is_some());
    }
}