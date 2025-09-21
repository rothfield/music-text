use crate::parse::model::{
    Document, DocumentElement, Stave, StaveLine, UpperLine, LowerLine, LyricsLine,
    UpperElement, LowerElement, Syllable, Source, Position, ContentLine, ContentElement, Beat, BeatElement, Note, SpatialAssignment, ConsumedElement
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

/// Consumed beat group with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedBeatGroup {
    pub start_pos: usize,
    pub end_pos: usize,
    pub original_source: Source,  // Source.value is now None
    pub underscore_count: usize,  // Number of underscores in the group
}

/// Consumed mordent with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedMordent {
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
            UpperElement::SlurIndicator { value, source } => {
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
            LowerElement::BeatGroupIndicator { value, source } => {
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
        "." => 1,  // single octave: +1/-1
        ":" => 2,  // double octave: +2/-2 (highest/lowest)
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

/// Consume mordents from upper line with move semantics
pub fn consume_mordents(
    mut upper_line: UpperLine
) -> (Vec<(usize, ConsumedMordent)>, UpperLine) {
    let mut consumed_mordents = Vec::new();
    let mut tracker = PositionTracker::new();

    // Consume mordents from upper line
    for element in &mut upper_line.elements {
        let pos = tracker.advance_for_upper_element(element);

        if let UpperElement::Mordent { source } = element {
            if let Some(value) = source.value.take() {
                let consumed = ConsumedMordent {
                    original_source: Source {
                        value: Some(value),
                        position: source.position.clone(),
                    },
                };
                consumed_mordents.push((pos, consumed));
            }
        }
    }

    (consumed_mordents, upper_line)
}

/// Assign consumed markers to content elements at exact positions
pub fn assign_markers_direct(
    mut content_elements: Vec<ParsedElement>,
    consumed_markers: Vec<(usize, ConsumedMarker)>
) -> (Vec<ParsedElement>, Vec<(usize, ConsumedMarker)>) {
    let mut remaining_markers = Vec::new();

    // Extract note positions (already 0-based)
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

/// Assign consumed mordents to content elements at exact positions
pub fn assign_mordents_direct(
    mut content_elements: Vec<ParsedElement>,
    consumed_mordents: Vec<(usize, ConsumedMordent)>
) -> (Vec<ParsedElement>, Vec<(usize, ConsumedMordent)>) {
    let mut remaining_mordents = Vec::new();

    // Extract note positions (already 0-based)
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
    for (mordent_pos, consumed_mordent) in consumed_mordents {
        let mut assigned = false;

        for (note_pos, note_idx) in &note_positions {
            if mordent_pos == *note_pos {
                // Transfer ownership of mordent to note
                if let ParsedElement::Note { children, .. } = &mut content_elements[*note_idx] {
                    // Add mordent ornament to note children
                    children.push(ParsedChild::Ornament {
                        kind: crate::rhythm::types::OrnamentType::Mordent,
                        distance: -1, // Above the note
                    });
                }
                assigned = true;
                break;
            }
        }

        if !assigned {
            remaining_mordents.push((mordent_pos, consumed_mordent));
        }
    }

    (content_elements, remaining_mordents)
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

/// Assign remaining mordents to closest notes
pub fn assign_mordents_nearest(
    mut content_elements: Vec<ParsedElement>,
    remaining_mordents: Vec<(usize, ConsumedMordent)>
) -> Vec<ParsedElement> {
    for (mordent_pos, _consumed_mordent) in remaining_mordents {
        let mut best_distance = usize::MAX;
        let mut best_note_idx = None;

        for (idx, element) in content_elements.iter().enumerate() {
            if let ParsedElement::Note { position, .. } = element {
                let note_pos = position.col;
                let distance = if mordent_pos > note_pos {
                    mordent_pos - note_pos
                } else {
                    note_pos - mordent_pos
                };

                if distance < best_distance {
                    best_distance = distance;
                    best_note_idx = Some(idx);
                }
            }
        }

        // Transfer ownership to best match
        if let Some(note_idx) = best_note_idx {
            if let ParsedElement::Note { children, .. } = &mut content_elements[note_idx] {
                // Add mordent ornament to note children
                children.push(ParsedChild::Ornament {
                    kind: crate::rhythm::types::OrnamentType::Mordent,
                    distance: -1, // Above the note
                });
            }
        }
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

        if let UpperElement::SlurIndicator { value, source } = element {
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

    // Assign slur types to all elements based on consumed slurs
    for consumed_slur in consumed_slurs {
        let element_positions: Vec<(usize, usize)> = content_elements.iter()
            .enumerate()
            .filter_map(|(idx, element)| {
                match element {
                    ParsedElement::Note { position, .. } => Some((position.col, idx)),
                    ParsedElement::Rest { position, .. } => Some((position.col, idx)),
                    ParsedElement::Dash { position, .. } => Some((position.col, idx)),
                    ParsedElement::Barline { position, .. } => Some((position.col, idx)),
                    ParsedElement::Whitespace { position, .. } => Some((position.col, idx)),
                    _ => None,
                }
            })
            .collect();

        for (element_pos, element_idx) in element_positions {
            if element_pos >= consumed_slur.start_pos && element_pos <= consumed_slur.end_pos {
                match &mut content_elements[element_idx] {
                    ParsedElement::Note { in_slur, .. } => {
                        *in_slur = true;
                    },
                    // Only notes have in_slur field, skip other elements
                    _ => {}
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
            UpperElement::SlurIndicator { source, .. } => {
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
            LowerElement::BeatGroupIndicator { source, .. } => {
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

/// Validate that all beat groups were properly processed
pub fn validate_beat_group_processing(remaining_beat_groups: &[(usize, ConsumedBeatGroup)]) -> Vec<String> {
    let mut warnings = Vec::new();

    for (pos, beat_group) in remaining_beat_groups {
        warnings.push(format!(
            "Unprocessed beat group indicator at position {} (span: {}-{}): {} underscores could not be assigned to notes",
            pos,
            beat_group.start_pos,
            beat_group.end_pos,
            beat_group.underscore_count
        ));
    }

    warnings
}

/// Consume beat groups from lower lines using move semantics with position tracking
fn consume_beat_groups(mut lower_lines: Vec<LowerLine>) -> (Vec<(usize, ConsumedBeatGroup)>, Vec<LowerLine>) {
    let mut consumed_groups = Vec::new();

    for lower_line in &mut lower_lines {
        let mut tracker = PositionTracker::new();

        for element in &mut lower_line.elements {
            let pos = tracker.advance_for_lower_element(element);

            if let LowerElement::BeatGroupIndicator { value, source } = element {
                if let Some(underscore_value) = source.value.take() {
                    let consumed = ConsumedBeatGroup {
                        start_pos: pos,
                        end_pos: pos + underscore_value.len() - 1,
                        underscore_count: underscore_value.len(),
                        original_source: Source {
                            value: Some(underscore_value),
                            position: source.position.clone(),
                        },
                    };
                    consumed_groups.push((pos, consumed));
                }
            }
        }
    }

    (consumed_groups, lower_lines)
}

/// Assign beat groups to content elements at exact positions
pub fn assign_beat_groups_direct(
    mut content_elements: Vec<ParsedElement>,
    consumed_beat_groups: Vec<(usize, ConsumedBeatGroup)>
) -> (Vec<ParsedElement>, Vec<(usize, ConsumedBeatGroup)>) {
    use crate::rhythm::types::BeatGroupRole;
    let mut remaining_beat_groups = Vec::new();

    // Extract element positions (convert from 1-based col to 0-based position for matching)
    let note_positions: Vec<(usize, usize)> = content_elements.iter()
        .enumerate()
        .filter_map(|(idx, element)| {
            match element {
                ParsedElement::Note { position, .. } => Some((position.col, idx)),
                ParsedElement::Rest { position, .. } => Some((position.col, idx)),
                ParsedElement::Dash { position, .. } => Some((position.col, idx)),
                ParsedElement::Barline { position, .. } => Some((position.col, idx)),
                ParsedElement::Whitespace { position, .. } => Some((position.col, idx)),
                _ => None,
            }
        })
        .collect();

    // Direct assignment pass
    for (beat_group_pos, consumed_beat_group) in consumed_beat_groups {
        // Find elements within beat group span
        let mut notes_in_group = Vec::new();

        for (note_pos, note_idx) in &note_positions {
            if *note_pos >= consumed_beat_group.start_pos && *note_pos <= consumed_beat_group.end_pos {
                notes_in_group.push(*note_idx);
            }
        }

        // Only assign if we have 2+ musical elements (minimum for a group)
        if notes_in_group.len() >= 2 {
            let mut assigned = false;

            // Assign beat group roles
            for (i, &element_index) in notes_in_group.iter().enumerate() {
                match &mut content_elements[element_index] {
                    ParsedElement::Note { beat_group, in_beat_group, children, position, .. } => {
                        // Check for overlap conflict with enhanced reporting
                        if beat_group.is_some() {
                            eprintln!("Warning: Beat group overlap detected at position {}:{} - existing assignment preserved", position.row, position.col);
                            // Don't overwrite existing assignment - add to remaining for fallback
                            continue;
                        }

                        let role = if i == 0 {
                            BeatGroupRole::Start
                        } else if i == notes_in_group.len() - 1 {
                            BeatGroupRole::End
                        } else {
                            BeatGroupRole::Middle
                        };

                        *beat_group = Some(role.clone());
                        *in_beat_group = true;

                        // Add beat group indicator to Start note's children
                        if matches!(role, BeatGroupRole::Start) {
                            use crate::rhythm::types::ParsedChild;
                            let beat_group_child = ParsedChild::BeatGroupIndicator {
                                symbol: consumed_beat_group.original_source.value.clone().unwrap_or_default(),
                                span: consumed_beat_group.underscore_count,
                            };
                            children.push(beat_group_child);
                        }

                        assigned = true;
                    },
                    // Only notes have in_beat_group field, others are counted but not modified
                    ParsedElement::Rest { .. } => {
                        assigned = true;
                    },
                    ParsedElement::Dash { .. } => {
                        assigned = true;
                    },
                    ParsedElement::Barline { .. } => {
                        assigned = true;
                    },
                    ParsedElement::Whitespace { .. } => {
                        assigned = true;
                    },
                    _ => {}
                }
            }

            if !assigned {
                remaining_beat_groups.push((beat_group_pos, consumed_beat_group));
            }
        } else {
            // Not enough elements for a group - add to remaining
            remaining_beat_groups.push((beat_group_pos, consumed_beat_group));
        }
    }

    (content_elements, remaining_beat_groups)
}

/// Assign remaining beat groups to closest available notes
pub fn assign_beat_groups_nearest(
    mut content_elements: Vec<ParsedElement>,
    remaining_beat_groups: Vec<(usize, ConsumedBeatGroup)>
) -> (Vec<ParsedElement>, Vec<(usize, ConsumedBeatGroup)>) {
    use crate::rhythm::types::BeatGroupRole;
    let mut still_remaining = Vec::new();

    for (beat_group_pos, consumed_beat_group) in remaining_beat_groups {
        // Find available elements (not already in beat groups) within a reasonable range
        let mut candidate_notes = Vec::new();

        for (idx, element) in content_elements.iter().enumerate() {
            let (position, already_assigned) = match element {
                ParsedElement::Note { beat_group, position, .. } => {
                    (position, beat_group.is_some())
                },
                ParsedElement::Rest { position, .. } => {
                    (position, false) // Rests don't have in_beat_group field
                },
                ParsedElement::Dash { position, .. } => {
                    (position, false) // Dashes don't have in_beat_group field
                },
                ParsedElement::Barline { position, .. } => {
                    (position, false) // Barlines don't have in_beat_group field
                },
                ParsedElement::Whitespace { position, .. } => {
                    (position, false) // Whitespace doesn't have in_beat_group field
                },
                _ => continue,
            };

            // Only consider elements without existing beat group assignments
            if !already_assigned {
                let element_pos = position.col;  // Already 0-based
                let distance = if beat_group_pos > element_pos {
                    beat_group_pos - element_pos
                } else {
                    element_pos - beat_group_pos
                };

                // Only consider elements within reasonable distance (e.g., 5 columns)
                if distance <= 5 {
                    candidate_notes.push((distance, element_pos, idx));
                }
            }
        }

        // Sort by distance, then by position
        candidate_notes.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        // Try to find at least 2 notes for a meaningful beat group
        if candidate_notes.len() >= 2 {
            // Take the closest available notes
            let selected_notes: Vec<usize> = candidate_notes.iter()
                .take(std::cmp::min(candidate_notes.len(), consumed_beat_group.underscore_count))
                .map(|(_, _, idx)| *idx)
                .collect();

            if selected_notes.len() >= 2 {
                // Assign beat group roles to selected elements
                for (i, &element_index) in selected_notes.iter().enumerate() {
                    match &mut content_elements[element_index] {
                        ParsedElement::Note { beat_group, in_beat_group, .. } => {
                            *beat_group = Some(if i == 0 {
                                BeatGroupRole::Start
                            } else if i == selected_notes.len() - 1 {
                                BeatGroupRole::End
                            } else {
                                BeatGroupRole::Middle
                            });
                            *in_beat_group = true;
                        },
                        // Only notes have in_beat_group field, others are counted but not modified
                        ParsedElement::Rest { .. } => {
                            // Rests don't have in_beat_group field
                        },
                        ParsedElement::Dash { .. } => {
                            // Dashes don't have in_beat_group field
                        },
                        ParsedElement::Barline { .. } => {
                            // Barlines don't have in_beat_group field
                        },
                        ParsedElement::Whitespace { .. } => {
                            // Whitespace doesn't have in_beat_group field
                        },
                        _ => {}
                    }
                }
            } else {
                // Not enough notes selected, add to remaining
                still_remaining.push((beat_group_pos, consumed_beat_group));
            }
        } else {
            // No suitable notes found, add to remaining
            still_remaining.push((beat_group_pos, consumed_beat_group));
        }
    }

    (content_elements, still_remaining)
}

/// Main spatial assignment processor
pub fn process_spatial_assignments(
    mut document: Document
) -> Result<(Document, Vec<String>), String> {
    let mut warnings = Vec::new();

    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            let (enhanced_stave, stave_warnings) = process_stave_spatial_old(stave)?;
            *stave = enhanced_stave;
            warnings.extend(stave_warnings);
        }
    }

    Ok((document, warnings))
}

/// Process spatial assignments for unified model (ContentLine with beats)
pub fn process_spatial_assignments_unified(
    mut document: Document
) -> Result<(Document, Vec<String>), String> {
    let mut warnings = Vec::new();

    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            let (enhanced_stave, stave_warnings) = process_stave_spatial_unified(stave)?;
            *stave = enhanced_stave;
            warnings.extend(stave_warnings);
        }
    }

    Ok((document, warnings))
}

/// Process spatial assignments for a single stave with unified model
fn process_stave_spatial_unified(stave: &mut Stave) -> Result<(Stave, Vec<String>), String> {
    let mut warnings = Vec::new();

    // Extract different line types
    let mut upper_lines = Vec::new();
    let mut lower_lines = Vec::new();
    let mut lyrics_lines = Vec::new();
    let mut content_line_indices = Vec::new();

    // Collect lines and find content line indices
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
            StaveLine::ContentLine(_) => {
                content_line_indices.push(idx);
            }
            _ => {} // Handle other line types as needed
        }
    }

    // Process spatial assignments for each content line
    for content_idx in content_line_indices {
        if let StaveLine::ContentLine(content_line) = &mut stave.lines[content_idx] {
            // Process spatial assignments for this content line
            let (updated_content_line, line_warnings) = process_content_line_spatial(
                content_line,
                &mut upper_lines,
                &mut lower_lines,
                &lyrics_lines
            )?;

            *content_line = updated_content_line;
            warnings.extend(line_warnings);
        }
    }

    Ok((stave.clone(), warnings))
}

/// Process spatial assignments for a single content line
fn process_content_line_spatial(
    content_line: &mut ContentLine,
    upper_lines: &mut [UpperLine],
    lower_lines: &mut [LowerLine],
    lyrics_lines: &[LyricsLine]
) -> Result<(ContentLine, Vec<String>), String> {
    let mut warnings = Vec::new();

    // For each beat in the content line, find notes and apply spatial assignments
    for element in &mut content_line.elements {
        if let ContentElement::Beat(beat) = element {
            for beat_element in &mut beat.elements {
                if let BeatElement::Note(note) = beat_element {
                    // Apply spatial assignments to this note
                    apply_spatial_assignments_to_note(note, upper_lines, lower_lines, lyrics_lines);
                }
            }
        }
    }

    Ok((content_line.clone(), warnings))
}

/// Apply spatial assignments from annotation lines to a specific note
fn apply_spatial_assignments_to_note(
    note: &mut Note,
    upper_lines: &mut [UpperLine],
    lower_lines: &mut [LowerLine],
    _lyrics_lines: &[LyricsLine]
) {
    let note_position = note.source.position.column;

    // Process octave markers from upper lines - use move semantics
    for upper_line in upper_lines {
        for element in &mut upper_line.elements {
            if let UpperElement::UpperOctaveMarker { marker, source } = element {
                if source.position.column == note_position {
                    // Move the complete element to consumed_elements
                    if let Some(marker_value) = source.value.take() {
                        let octave_value = match marker.as_str() {
                            "." => 1,
                            ":" => 2,
                            _ => 0,
                        };

                        // Store complete consumed element directly
                        note.consumed_elements.push(ConsumedElement::UpperOctaveMarker {
                            source: Source {
                                value: Some(marker_value),
                                position: source.position.clone(),
                            },
                        });

                        // Apply octave modification
                        note.octave += octave_value;
                    }
                }
            }
        }
    }

    // Process octave markers from lower lines - use move semantics
    for lower_line in lower_lines {
        for element in &mut lower_line.elements {
            if let LowerElement::LowerOctaveMarker { marker, source } = element {
                if source.position.column == note_position {
                    // Move the complete element to consumed_elements
                    if let Some(marker_value) = source.value.take() {
                        let octave_value = match marker.as_str() {
                            "." => -1,
                            ":" => -2,
                            _ => 0,
                        };

                        // Store complete consumed element directly
                        note.consumed_elements.push(ConsumedElement::LowerOctaveMarker {
                            source: Source {
                                value: Some(marker_value),
                                position: source.position.clone(),
                            },
                        });

                        // Apply octave modification
                        note.octave += octave_value;
                    }
                }
            }
        }
    }

    // TODO: Add slur, syllable, beat group, and mordent processing
}

/// Process spatial assignments for a single stave (OLD VERSION - ParsedElement based)
fn process_stave_spatial_old(stave: &mut Stave) -> Result<(Stave, Vec<String>), String> {
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
            StaveLine::Content(_) | StaveLine::ContentLine(_) => {
                content_line_index = Some(idx);
            }
            _ => {} // Handle other line types as needed
        }
    }

    // Process spatial assignments if we have lower line and content line
    if !lower_lines.is_empty() && content_line_index.is_some() {
        let content_idx = content_line_index.unwrap();

        // Extract content elements from the content line
        match &mut stave.lines[content_idx] {
            StaveLine::Content(content_elements) => {
            // Create empty upper line if none exists
            let upper_line = if upper_lines.is_empty() {
                UpperLine {
                    elements: vec![],
                    source: Source {
                        value: Some("".to_string()),
                        position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
                    },
                }
            } else {
                upper_lines[0].clone()
            };

            let lower_line = lower_lines[0].clone();

            // 1. Consume octave markers
            let (consumed_markers, updated_upper, updated_lower) =
                consume_octave_markers(upper_line, lower_line);


            // 2. Assign markers to content elements
            let (content_with_octaves, remaining_markers) =
                assign_markers_direct(content_elements.clone(), consumed_markers);

            // 3. Assign remaining markers to nearest notes
            let content_with_octaves_and_markers = assign_markers_nearest(content_with_octaves, remaining_markers);

            // 4. Consume beat groups from lower lines (use already updated lower line)
            let (consumed_beat_groups, final_updated_lower_lines) = consume_beat_groups(vec![updated_lower]);

            // 5. Assign beat groups to content elements with two-phase approach
            let (content_with_beat_groups, remaining_beat_groups) =
                assign_beat_groups_direct(content_with_octaves_and_markers, consumed_beat_groups);

            // 6. Assign remaining beat groups to nearest notes
            let (final_content, still_remaining_beat_groups) = assign_beat_groups_nearest(content_with_beat_groups, remaining_beat_groups);

            // 7. Validate beat group processing and collect warnings
            let beat_group_warnings = validate_beat_group_processing(&still_remaining_beat_groups);
            warnings.extend(beat_group_warnings);

            // Update the content line with processed elements
            *content_elements = final_content;

            // Update the stave with consumed upper and lower lines
            // Find and update upper line
            for line in &mut stave.lines {
                if let StaveLine::Upper(_) = line {
                    *line = StaveLine::Upper(updated_upper);
                    break;
                }
            }

            // Find and update lower line(s)
            let mut lower_line_idx = 0;
            for line in &mut stave.lines {
                if let StaveLine::Lower(_) = line {
                    if lower_line_idx < final_updated_lower_lines.len() {
                        *line = StaveLine::Lower(final_updated_lower_lines[lower_line_idx].clone());
                        lower_line_idx += 1;
                    }
                }
            }
            }
            StaveLine::ContentLine(_content_line) => {
                // ContentLine processing is handled by the unified version
                // This old function is kept for backward compatibility with ParsedElement-based processing
            }
            _ => {}
        }
    }

    Ok((stave.clone(), warnings))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::model::{PitchCode, NotationSystem};

    fn create_test_note(pitch: &str, column: usize) -> Note {
        Note {
            source: Source {
                value: Some(pitch.to_string()),
                position: Position { line: 1, column, index_in_line: (column.saturating_sub(1)), index_in_doc: 0 },
            },
            octave: 0,
            pitch_code: PitchCode::N1,
            notation_system: NotationSystem::Number,
            spatial_assignments: Vec::new(),
            consumed_elements: Vec::new(),
            duration: None,
        }
    }

    fn create_test_upper_line_with_markers() -> UpperLine {
        UpperLine {
            elements: vec![
                UpperElement::UpperOctaveMarker {
                    marker: ".".to_string(),
                    source: Source {
                        value: Some(".".to_string()),
                        position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
                    },
                },
                UpperElement::Space {
                    count: 2,
                    source: Source {
                        value: Some("  ".to_string()),
                        position: Position { line: 0, column: 1, index_in_line: 0, index_in_doc: 0 },
                    },
                },
                UpperElement::UpperOctaveMarker {
                    marker: ":".to_string(),
                    source: Source {
                        value: Some(":".to_string()),
                        position: Position { line: 0, column: 3, index_in_line: 2, index_in_doc: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".  :".to_string()),
                position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
            },
        }
    }

    fn create_test_lower_line_empty() -> LowerLine {
        LowerLine {
            elements: vec![],
            source: Source {
                value: Some("".to_string()),
                position: Position { line: 2, column: 0, index_in_line: 0, index_in_doc: 0 },
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
                        position: Position { line: 2, column: 0, index_in_line: 0, index_in_doc: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".".to_string()),
                position: Position { line: 2, column: 0, index_in_line: 0, index_in_doc: 0 },
            },
        }
    }

    #[test]
    fn test_octave_marker_to_number() {
        // Test new grammar: . = +1/-1, : = +2/-2
        assert_eq!(octave_marker_to_number(".", true), 1);   // upper_octave_marker
        assert_eq!(octave_marker_to_number(":", true), 2);   // highest_octave_marker
        assert_eq!(octave_marker_to_number(".", false), -1); // lower_octave_marker
        assert_eq!(octave_marker_to_number(":", false), -2); // lowest_octave_marker

        // Test invalid markers (removed from grammar)
        assert_eq!(octave_marker_to_number("*", true), 0);   // removed from grammar
        assert_eq!(octave_marker_to_number("'", true), 0);   // removed from grammar
        assert_eq!(octave_marker_to_number("x", true), 0);   // invalid marker
    }

    #[test]
    fn test_position_tracker() {
        let mut tracker = PositionTracker::new();

        let space_element = UpperElement::Space {
            count: 3,
            source: Source {
                value: Some("   ".to_string()),
                position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
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
                    position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
                },
                marker_symbol: ".".to_string(),
                is_upper: true,
            }),
            (3, ConsumedMarker {
                octave_value: 2,
                original_source: Source {
                    value: Some(":".to_string()),
                    position: Position { line: 0, column: 3, index_in_line: 2, index_in_doc: 0 },
                },
                marker_symbol: ":".to_string(),
                is_upper: true,
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
                    position: Position { line: 0, column: 2, index_in_line: 1, index_in_doc: 0 },
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
                        position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
                    },
                },
                UpperElement::SlurIndicator {
                    value: "___".to_string(),
                    source: Source {
                        value: Some("___".to_string()),
                        position: Position { line: 0, column: 2, index_in_line: 1, index_in_doc: 0 },
                    },
                },
            ],
            source: Source {
                value: Some("  ___".to_string()),
                position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
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
            if let UpperElement::SlurIndicator { source, .. } = element {
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
                        position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".".to_string()),
                position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
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
                        position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
                    },
                },
            ],
            source: Source {
                value: Some(".".to_string()),
                position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
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
                position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
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
                    position: Position { line: 2, column: 0, index_in_line: 0, index_in_doc: 0 },
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
            position: Position { line: 0, column: 0, index_in_line: 0, index_in_doc: 0 },
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
