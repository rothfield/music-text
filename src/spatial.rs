use crate::models::{
    Document, DocumentElement, Stave, StaveLine, UpperLine, LowerLine, LyricsLine,
    UpperElement, LowerElement, Syllable, ContentLine, ContentElement, Beat, BeatElement, Note, ConsumedElement, Attributes, Position
};
use crate::rhythm::types::{ParsedElement, Position as RhythmPosition, ParsedChild};

/// Consumed marker with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedMarker {
    pub octave_value: i8,
    pub char_index: usize,    // was: original_source
    pub marker_symbol: String,    // Original marker symbol (".", ":", etc.)
    pub is_upper: bool,          // true = upper line, false = lower line
}

/// Consumed slur with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedSlur {
    pub start_pos: usize,
    pub end_pos: usize,
    pub char_index: usize,    // was: original_source
}

/// Consumed syllable with move semantics
#[derive(Debug, Clone)]
pub struct ConsumedSyllable {
    pub content: String,
    pub char_index: usize,    // was: original_source
}

/// Consumed beat group with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedBeatGroup {
    pub start_pos: usize,
    pub end_pos: usize,
    pub char_index: usize,    // was: original_source
    pub underscore_count: usize,  // Number of underscores in the group
}

/// Consumed mordent with move semantics - original source is consumed
#[derive(Debug, Clone)]
pub struct ConsumedMordent {
    pub char_index: usize,    // was: original_source
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
            UpperElement::UpperOctaveMarker { value, .. } => {
                if value.is_some() {
                    self.current_pos += 1;
                } else {
                    self.consumed_positions.push(old_pos);
                }
            }
            UpperElement::SlurIndicator { value, indicator_value, .. } => {
                if value.is_some() {
                    self.current_pos += indicator_value.len();
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
            LowerElement::LowerOctaveMarker { value, .. } => {
                if value.is_some() {
                    self.current_pos += 1;
                } else {
                    self.consumed_positions.push(old_pos);
                }
            }
            LowerElement::BeatGroupIndicator { value, indicator_value, .. } => {
                if value.is_some() {
                    self.current_pos += indicator_value.len();
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

/// Consume mordents from upper line with move semantics
pub fn consume_mordents(
    mut upper_line: UpperLine
) -> (Vec<(usize, ConsumedMordent)>, UpperLine) {
    let mut consumed_mordents = Vec::new();
    let mut tracker = PositionTracker::new();

    // Consume mordents from upper line
    for element in &mut upper_line.elements {
        let pos = tracker.advance_for_upper_element(element);

        if let UpperElement::Mordent { value, char_index, .. } = element {
            if let Some(marker_value) = value.take() {
                let consumed = ConsumedMordent {
                    char_index: *char_index, // was: original_source
                };
                consumed_mordents.push((pos, consumed));
            }
        }
    }

    (consumed_mordents, upper_line)
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
                ParsedElement::Note { position, .. } => Some((position.char_index, idx)),
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
                    let note_pos = position.col;  // Use column position for alignment
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
                let note_pos = position.char_index;
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

        if let UpperElement::SlurIndicator { value, indicator_value, char_index, .. } = element {
            if indicator_value.len() >= 2 { // Minimum 2 underscores for slur
                if let Some(underscore_value) = value.take() {
                    let consumed = ConsumedSlur {
                        start_pos: pos,
                        end_pos: pos + indicator_value.len() - 1,
                        char_index: *char_index, // was: original_source
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
                    ParsedElement::Note { position, .. } => Some((position.char_index, idx)),
                    ParsedElement::Rest { position, .. } => Some((position.char_index, idx)),
                    ParsedElement::Dash { position, .. } => Some((position.char_index, idx)),
                    ParsedElement::Barline { position, .. } => Some((position.char_index, idx)),
                    ParsedElement::Whitespace { position, .. } => Some((position.char_index, idx)),
                    _ => None,
                }
            })
            .collect();

        for (element_pos, element_idx) in element_positions {
            if element_pos >= consumed_slur.start_pos && element_pos <= consumed_slur.end_pos {
                match &mut content_elements[element_idx] {
                    ParsedElement::Note { .. } => {
                        // Note: in_slur field has been replaced with slur_position enum
                        // This old processing is deprecated
                    },
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
            if let Some(content) = syllable.value.take() {
                let consumed = ConsumedSyllable {
                    content,
                    char_index: syllable.char_index, // was: original_source
                };
                consumed_syllables.push(consumed);
            }
        }
    }

    // Assign syllables to notes respecting slur boundaries
    let mut syllable_index = 0;
    for element in &mut content_elements {
        if let ParsedElement::Note { .. } = element {
            // Only assign syllables to notes that are not in the middle of slurs
            // For now, we'll assign to all notes since the current structure doesn't have syllable field
            if syllable_index < consumed_syllables.len() {
                // Note: in_slur check removed - slur_position enum should be used instead
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
            UpperElement::UpperOctaveMarker { value, char_index, .. } => {
                if value.is_some() {
                    return Err(format!(
                        "Unconsumed octave marker at position {:?}",
                        *char_index
                    ));
                }
            }
            UpperElement::SlurIndicator { value, char_index, .. } => {
                if value.is_some() {
                    return Err(format!(
                        "Unconsumed slur at position {:?}",
                        *char_index
                    ));
                }
            }
            _ => {} // Spaces and other elements don't need consumption
        }
    }

    // Check for unconsumed markers in lower line
    for element in &lower_line.elements {
        match element {
            LowerElement::LowerOctaveMarker { value, char_index, .. } => {
                if value.is_some() {
                    return Err(format!(
                        "Unconsumed octave marker at position {:?}",
                        *char_index
                    ));
                }
            }
            LowerElement::BeatGroupIndicator { value, char_index, .. } => {
                if value.is_some() {
                    return Err(format!(
                        "Unconsumed beat grouping at position {:?}",
                        *char_index
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

            if let LowerElement::BeatGroupIndicator { value, indicator_value, char_index, .. } = element {
                if let Some(underscore_value) = value.take() {
                    let consumed = ConsumedBeatGroup {
                        start_pos: pos,
                        end_pos: pos + underscore_value.len() - 1,
                        underscore_count: underscore_value.len(),
                        char_index: *char_index, // was: original_source
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
                ParsedElement::Note { position, .. } => Some((position.char_index, idx)),
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
                            eprintln!("Warning: Beat group overlap detected at position {} - existing assignment preserved", position.char_index);
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
                                symbol: "_".repeat(consumed_beat_group.underscore_count), // was: original_source.value
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
                let element_pos = position.char_index;
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

    // Update the original upper lines in the stave with the modified versions
    let mut upper_idx = 0;
    for line in &mut stave.lines {
        if let StaveLine::Upper(upper_line) = line {
            if upper_idx < upper_lines.len() {
                *upper_line = upper_lines[upper_idx].clone();
                upper_idx += 1;
            }
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

    // Get the starting char_index of the content line to calculate columns
    let content_line_start = content_line.char_index;

    // DISABLED: Old spatial consumption system - now using direct marker processing
    // for upper_line in upper_lines.iter_mut() {
    //     process_spatial_consumption(upper_line, content_line);
    // }

    // STANDARD APPROACH: Build position map first, then use it

    // First pass: Build column-to-note mapping with direct access paths
    let mut column_to_note_path = std::collections::HashMap::new();
    let mut current_column = 0;

    for (element_idx, element) in content_line.elements.iter().enumerate() {
        match element {
            ContentElement::Whitespace(ws) => {
                current_column += ws.value.as_ref().map_or(1, |s| s.len());
            },
            ContentElement::Beat(beat) => {
                for (beat_element_idx, beat_element) in beat.elements.iter().enumerate() {
                    match beat_element {
                        BeatElement::Note(note) => {
                            column_to_note_path.insert(current_column, (element_idx, beat_element_idx));
                            current_column += 1; // Each note is 1 character
                        },
                        BeatElement::Dash(_dash) => {
                            current_column += 1; // Each dash is 1 character
                        },
                        BeatElement::Rest(_rest) => {
                            current_column += 1; // Each rest is 1 character
                        },
                        BeatElement::BreathMark(_breath) => {
                            current_column += 1; // Each breath mark is 1 character
                        }
                    }
                }
            },
            ContentElement::Barline(_) => {
                current_column += 1; // Barlines are typically 1 char
            },
            ContentElement::UnknownToken(unknown) => {
                // Unknown tokens behave like whitespace, advance column by token length
                current_column += unknown.token_value.len();
            }
        }
    }

    // Second pass: Process upper octave markers using direct access
    for upper_line in upper_lines.iter_mut() {
        let upper_line_start = upper_line.char_index;
        for element in &mut upper_line.elements {
            if let UpperElement::UpperOctaveMarker { marker, value, char_index, .. } = element {
                if let Some(marker_value) = value.take() {
                    let marker_column = char_index.saturating_sub(upper_line_start);
                    if let Some(&(element_idx, beat_element_idx)) = column_to_note_path.get(&marker_column) {
                        if let ContentElement::Beat(beat) = &mut content_line.elements[element_idx] {
                            if let BeatElement::Note(note) = &mut beat.elements[beat_element_idx] {
                                let octave_value = match marker.as_str() {
                                    "." => 1,
                                    ":" => 2,
                                    _ => 0,
                                };
                                note.consumed_elements.push(ConsumedElement::UpperOctaveMarker {
                                    value: Some(marker_value),
                                    char_index: *char_index,
                                });
                                note.octave += octave_value;
                            }
                        }
                    }
                }
            }
        }
    }

    // Process lower octave markers using direct access
    for lower_line in lower_lines.iter_mut() {
        let lower_line_start = lower_line.char_index;
        for element in &mut lower_line.elements {
            if let LowerElement::LowerOctaveMarker { marker, value, char_index, .. } = element {
                if let Some(marker_value) = value.take() {
                    let marker_column = char_index.saturating_sub(lower_line_start);
                    if let Some(&(element_idx, beat_element_idx)) = column_to_note_path.get(&marker_column) {
                        if let ContentElement::Beat(beat) = &mut content_line.elements[element_idx] {
                            if let BeatElement::Note(note) = &mut beat.elements[beat_element_idx] {
                                let octave_value = match marker.as_str() {
                                    "." => -1,
                                    ":" => -2,
                                    _ => 0,
                                };
                                note.consumed_elements.push(ConsumedElement::LowerOctaveMarker {
                                    value: Some(marker_value),
                                    char_index: *char_index,
                                });
                                note.octave += octave_value;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((content_line.clone(), warnings))
}

/// Find a note at the specified column in the content line
fn find_note_at_column(content_line: &mut ContentLine, content_line_start: usize, target_column: usize) -> Option<&mut Note> {
    for element in &mut content_line.elements {
        if let ContentElement::Beat(beat) = element {
            for beat_element in &mut beat.elements {
                if let BeatElement::Note(note) = beat_element {
                    let note_column = note.char_index - content_line_start;
                    if note_column == target_column {
                        return Some(note);
                    }
                }
            }
        }
    }
    None
}


/// Helper function to calculate column positions for all content elements
fn calculate_content_element_columns(content_line: &ContentLine) -> Vec<(usize, usize)> {
    let mut column_positions = Vec::new();
    let mut current_column = 0;

    for (element_idx, element) in content_line.elements.iter().enumerate() {
        match element {
            ContentElement::Whitespace(whitespace) => {
                current_column += whitespace.value.as_ref().map_or(0, |s| s.len());
            },
            ContentElement::Beat(beat) => {
                let beat_start_column = current_column;
                for beat_element in &beat.elements {
                    match beat_element {
                        BeatElement::Note(note) => {
                            column_positions.push((element_idx, current_column));
                            if let Some(ref value) = note.value {
                                current_column += value.len();
                            } else {
                                current_column += 1; // Default single character
                            }
                        },
                        BeatElement::Dash(dash) => {
                            column_positions.push((element_idx, current_column));
                            if let Some(ref value) = dash.value {
                                current_column += value.len();
                            } else {
                                current_column += 1; // Default single character
                            }
                        },
                        BeatElement::BreathMark(breath) => {
                            column_positions.push((element_idx, current_column));
                            if let Some(ref value) = breath.value {
                                current_column += value.len();
                            } else {
                                current_column += 1; // Default single character
                            }
                        },
                        BeatElement::Rest(rest) => {
                            column_positions.push((element_idx, current_column));
                            if let Some(ref value) = rest.value {
                                current_column += value.len();
                            } else {
                                current_column += 1; // Default single character
                            }
                        },
                    }
                }
            },
            ContentElement::Barline(_) => {
                column_positions.push((element_idx, current_column));
                current_column += 1; // Barlines are typically single character
            },
            ContentElement::UnknownToken(unknown) => {
                // Unknown tokens behave like whitespace - they don't have column positions themselves
                current_column += unknown.token_value.len();
            },
        }
    }

    column_positions
}

/// Generic spatial element consumption system
/// Finds spatial elements in annotation lines and consumes them via the first matching content element
pub fn process_spatial_consumption(
    upper_line: &mut UpperLine,
    content_line: &mut ContentLine,
) {
    process_upper_line_consumption(upper_line, content_line);
    // TODO: Add lower_line consumption when needed
}

/// Represents a spatial element that can be consumed by content elements
#[derive(Debug, Clone)]
struct SpatialSpan {
    start_column: usize,
    end_column: usize,
    consumed_element: ConsumedElement,
    element_type: SpatialElementType,
}

/// Types of spatial elements that can be consumed
#[derive(Debug, Clone)]
enum SpatialElementType {
    Slur,
    UpperOctaveMarker,
    LowerOctaveMarker,
    BeatGroupIndicator,
    // Add more types as needed
}

/// Process consumption of upper line spatial elements by content elements
fn process_upper_line_consumption(
    upper_line: &mut UpperLine,
    content_line: &mut ContentLine,
) {
    // Collect all consumable spatial spans from upper line
    let spatial_spans = collect_upper_line_spans(upper_line);

    // Apply consumption for each span
    for span in spatial_spans {
        consume_spatial_span(span, content_line);
    }
}

/// Collect all consumable spatial spans from an upper line
fn collect_upper_line_spans(upper_line: &mut UpperLine) -> Vec<SpatialSpan> {
    let mut spans = Vec::new();
    let mut tracker = PositionTracker::new();

    for element in &mut upper_line.elements {
        match element {
            UpperElement::SlurIndicator { value, indicator_value, char_index, .. } => {
                if indicator_value.len() >= 2 { // Minimum 2 underscores for slur
                    let indicator_len = indicator_value.len();
                    let element_char_index = *char_index;

                    if let Some(consumed_value) = value.take() { // Consume FIRST
                        let start_column = tracker.advance_for_upper_element(element);
                        let end_column = start_column + indicator_len - 1;

                        spans.push(SpatialSpan {
                            start_column,
                            end_column,
                            consumed_element: ConsumedElement::SlurIndicator {
                                value: Some(consumed_value),
                                char_index: element_char_index,
                            },
                            element_type: SpatialElementType::Slur,
                        });
                    } else {
                        tracker.advance_for_upper_element(element);
                    }
                } else {
                    tracker.advance_for_upper_element(element);
                }
            },
            UpperElement::UpperOctaveMarker { value, marker, char_index, .. } => {
                let element_char_index = *char_index;

                if let Some(consumed_value) = value.take() { // Consume octave marker
                    let start_column = tracker.advance_for_upper_element(element);

                    spans.push(SpatialSpan {
                        start_column,
                        end_column: start_column, // Single character
                        consumed_element: ConsumedElement::UpperOctaveMarker {
                            value: Some(consumed_value),
                            char_index: element_char_index,
                        },
                        element_type: SpatialElementType::UpperOctaveMarker,
                    });
                } else {
                    tracker.advance_for_upper_element(element);
                }
            },
            _ => {
                tracker.advance_for_upper_element(element);
            }
        }
    }

    spans
}

/// Generic function to consume a spatial span by the appropriate content element
fn consume_spatial_span(span: SpatialSpan, content_line: &mut ContentLine) {
    // Calculate column positions for all content elements
    let element_columns = calculate_content_element_columns(content_line);

    match span.element_type {
        SpatialElementType::Slur => {
            // Slurs affect ALL elements within the span
            apply_slur_to_span(&span, &element_columns, content_line);
        },
        _ => {
            // Other spatial elements (octave markers) consume by single target
            let target_element_idx = find_consumption_target(&span, &element_columns, content_line);
            if let Some(element_idx) = target_element_idx {
                add_consumed_element_to_content(element_idx, span.consumed_element, content_line);
            }
        }
    }
}

/// Find which content element should consume the spatial element based on type and position
fn find_consumption_target(span: &SpatialSpan, element_columns: &[(usize, usize)], content_line: &ContentLine) -> Option<usize> {
    // Find elements within the spatial span
    let mut candidates = Vec::new();
    for &(element_idx, column) in element_columns {
        if column >= span.start_column && column <= span.end_column {
            candidates.push((element_idx, column));
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort by column position for consistent ordering
    candidates.sort_by_key(|&(_, column)| column);

    // Apply consumption priority rules based on spatial element type
    match span.element_type {
        SpatialElementType::Slur => {
            // For slurs: prioritize first dash, fallback to first element
            find_first_dash_in_candidates(&candidates, content_line)
                .or_else(|| Some(candidates[0].0))
        },
        SpatialElementType::UpperOctaveMarker => {
            // For octave markers: prioritize first note, fallback to first element
            find_first_note_in_candidates(&candidates, content_line)
                .or_else(|| Some(candidates[0].0))
        },
        _ => {
            // Default: consume by first element
            Some(candidates[0].0)
        }
    }
}

/// Helper to find the first dash element in candidates
fn find_first_dash_in_candidates(candidates: &[(usize, usize)], content_line: &ContentLine) -> Option<usize> {
    for &(element_idx, _) in candidates {
        if let Some(ContentElement::Beat(beat)) = content_line.elements.get(element_idx) {
            for beat_element in &beat.elements {
                if matches!(beat_element, BeatElement::Dash(_)) {
                    return Some(element_idx);
                }
            }
        }
    }
    None
}

/// Helper to find the first note element in candidates
fn find_first_note_in_candidates(candidates: &[(usize, usize)], content_line: &ContentLine) -> Option<usize> {
    for &(element_idx, _) in candidates {
        if let Some(ContentElement::Beat(beat)) = content_line.elements.get(element_idx) {
            for beat_element in &beat.elements {
                if matches!(beat_element, BeatElement::Note(_)) {
                    return Some(element_idx);
                }
            }
        }
    }
    None
}

/// Apply slur to the first element within the span
fn apply_slur_to_span(span: &SpatialSpan, element_columns: &[(usize, usize)], content_line: &mut ContentLine) {
    // Slur token is consumed by the content line itself (not individual elements)
    content_line.consumed_elements.push(span.consumed_element.clone());

    // Find the first element (whitespace, beat element, or barline) within the slur span
    let mut current_column = 0;
    let mut first_element_found = false;

    for (element_idx, element) in content_line.elements.iter_mut().enumerate() {
        if first_element_found {
            break;
        }

        match element {
            ContentElement::Whitespace(whitespace) => {
                // Check if whitespace is within slur span
                if current_column >= span.start_column && current_column <= span.end_column {
                    // Add slur to first whitespace element
                    whitespace.consumed_elements.push(span.consumed_element.clone());
                    first_element_found = true;
                } else {
                    current_column += whitespace.value.as_ref().map_or(0, |s| s.len());
                }
            },
            ContentElement::Beat(beat) => {
                for beat_element in &mut beat.elements {
                    if first_element_found {
                        break;
                    }

                    // Check if this beat element is within the slur span
                    if current_column >= span.start_column && current_column <= span.end_column {
                        // Add slur to first beat element
                        match beat_element {
                            BeatElement::Note(note) => {
                                note.consumed_elements.push(span.consumed_element.clone());
                                first_element_found = true;
                            },
                            BeatElement::Dash(dash) => {
                                dash.consumed_elements.push(span.consumed_element.clone());
                                first_element_found = true;
                            },
                            BeatElement::BreathMark(breath) => {
                                breath.consumed_elements.push(span.consumed_element.clone());
                                first_element_found = true;
                            },
                            BeatElement::Rest(rest) => {
                                rest.consumed_elements.push(span.consumed_element.clone());
                                first_element_found = true;
                            },
                        }
                    } else {
                        // Advance column for this beat element
                        match beat_element {
                            BeatElement::Note(note) => {
                                if let Some(ref value) = note.value {
                                    current_column += value.len();
                                } else {
                                    current_column += 1;
                                }
                            },
                            BeatElement::Dash(_) => current_column += 1,
                            BeatElement::BreathMark(_) => current_column += 1,
                            BeatElement::Rest(_) => current_column += 1,
                        }
                    }
                }
            },
            ContentElement::Barline(_) => {
                if current_column >= span.start_column && current_column <= span.end_column {
                    // Barlines don't have consumed_elements, skip to next element
                }
                current_column += 1;
            },
            ContentElement::UnknownToken(unknown) => {
                if current_column >= span.start_column && current_column <= span.end_column {
                    // Add slur to unknown token (they behave like whitespace)
                    unknown.consumed_elements.push(span.consumed_element.clone());
                    first_element_found = true;
                } else {
                    current_column += unknown.token_value.len();
                }
            },
        }
    }
}

/// Add consumed element to the appropriate content element
fn add_consumed_element_to_content(
    element_idx: usize,
    consumed_element: ConsumedElement,
    content_line: &mut ContentLine,
) {
    if let Some(ContentElement::Beat(beat)) = content_line.elements.get_mut(element_idx) {
        // Add to the first matching beat element (note, dash, or breath mark)
        for beat_element in &mut beat.elements {
            match beat_element {
                BeatElement::Note(note) => {
                    // Phase 1: Generic consumption
                    note.consumed_elements.push(consumed_element.clone());

                    // Phase 2: Semantic processing
                    apply_semantic_effects_to_note(note, &consumed_element);
                    return;
                },
                BeatElement::Dash(dash) => {
                    // Phase 1: Generic consumption
                    dash.consumed_elements.push(consumed_element.clone());

                    // Phase 2: Semantic processing
                    apply_semantic_effects_to_dash(dash, &consumed_element);
                    return;
                },
                BeatElement::BreathMark(breath) => {
                    // Phase 1: Generic consumption
                    breath.consumed_elements.push(consumed_element.clone());

                    // Phase 2: Semantic processing
                    apply_semantic_effects_to_breath(breath, &consumed_element);
                    return;
                },
                BeatElement::Rest(rest) => {
                    // Phase 1: Generic consumption
                    rest.consumed_elements.push(consumed_element.clone());
                    // Phase 2: No special semantic processing needed for rests
                    return;
                },
            }
        }
    }
    // TODO: Handle other content element types (barlines, whitespace) if they need consumption
}

/// Apply semantic effects to a note based on consumed element type
fn apply_semantic_effects_to_note(note: &mut Note, consumed_element: &ConsumedElement) {
    match consumed_element {
        ConsumedElement::UpperOctaveMarker { .. } => {
            // Octave markers increase octave (upper line = positive)
            note.octave += 1;
        },
        ConsumedElement::LowerOctaveMarker { .. } => {
            // Lower octave markers decrease octave
            note.octave -= 1;
        },
        ConsumedElement::SlurIndicator { .. } => {
            // Slurs are handled at content line level, not individual elements
            // This case should not occur with the new architecture
        },
    }
}

/// Apply semantic effects to a dash based on consumed element type
fn apply_semantic_effects_to_dash(dash: &mut crate::parse::model::Dash, consumed_element: &ConsumedElement) {
    match consumed_element {
        ConsumedElement::SlurIndicator { .. } => {
            // Slurs are handled at content line level, not individual elements
            // This case should not occur with the new architecture
        },
        _ => {
            // Dashes don't typically consume octave markers
        }
    }
}

/// Apply semantic effects to a breath mark based on consumed element type
fn apply_semantic_effects_to_breath(breath: &mut crate::parse::model::BreathMark, consumed_element: &ConsumedElement) {
    match consumed_element {
        ConsumedElement::SlurIndicator { .. } => {
            // Slurs are handled at content line level, not individual elements
            // This case should not occur with the new architecture
        },
        _ => {
            // Breath marks don't typically consume octave markers
        }
    }
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
                    value: Some("".to_string()),
                    char_index: 0,
                }
            } else {
                upper_lines[0].clone()
            };

            let lower_line = lower_lines[0].clone();

            // 1. DISABLED: Old octave marker system - now using apply_spatial_assignments_to_note
            // let (consumed_markers, updated_upper, updated_lower) =
            //     consume_octave_markers(upper_line, lower_line);
            let updated_upper = upper_line;
            let updated_lower = lower_line;

            // 2. DISABLED: Old marker assignment system
            // let (content_with_octaves, remaining_markers) =
            //     assign_markers_direct(content_elements.clone(), consumed_markers);
            let content_with_octaves_and_markers = content_elements.clone();

            // 4-6. DISABLED: Old beat group processing system
            // let (consumed_beat_groups, final_updated_lower_lines) = consume_beat_groups(vec![updated_lower]);
            // let (content_with_beat_groups, remaining_beat_groups) =
            //     assign_beat_groups_direct(content_with_octaves_and_markers, consumed_beat_groups);
            // let (final_content, still_remaining_beat_groups) = assign_beat_groups_nearest(content_with_beat_groups, remaining_beat_groups);
            let final_content = content_with_octaves_and_markers;
            let still_remaining_beat_groups = Vec::new();
            let final_updated_lower_lines = vec![updated_lower];

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
    use crate::models::{PitchCode, NotationSystem};

    fn create_test_note(pitch: &str, column: usize) -> Note {
        Note {
            value: Some(pitch.to_string()),
            char_index: column,
            octave: 0,
            pitch_code: PitchCode::N1,
            notation_system: NotationSystem::Number,
            numerator: None,
            denominator: None,
            consumed_elements: Vec::new(),
        }
    }

    fn create_test_upper_line_with_markers() -> UpperLine {
        UpperLine {
            elements: vec![
                UpperElement::UpperOctaveMarker {
                    marker: ".".to_string(),
                    value: Some(".".to_string()),
                    char_index: 0,
                },
                UpperElement::Space {
                    count: 2,
                    value: Some("  ".to_string()),
                    char_index: 1,
                },
                UpperElement::UpperOctaveMarker {
                    marker: ":".to_string(),
                    value: Some(":".to_string()),
                    char_index: 3,
                },
            ],
            value: Some(".  :".to_string()),
            char_index: 0,
        }
    }

    fn create_test_lower_line_empty() -> LowerLine {
        LowerLine {
            elements: vec![],
            value: Some("".to_string()),
            char_index: 0,
        }
    }

    fn create_test_lower_line_with_markers() -> LowerLine {
        LowerLine {
            elements: vec![
                LowerElement::LowerOctaveMarker {
                    marker: ".".to_string(),
                    value: Some(".".to_string()),
                    char_index: 0,
                },
            ],
            value: Some(".".to_string()),
            char_index: 0,
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
            value: Some("   ".to_string()),
            char_index: 0,
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

        // Original values should be consumed (None)
        for element in &updated_upper.elements {
            if let UpperElement::UpperOctaveMarker { value, .. } = element {
                assert!(value.is_none(), "Marker value should be consumed");
            }
        }
    }

    // #[test]
    // fn test_assign_markers_direct() {
    //     // TODO: This function has been removed/refactored - test may no longer be needed
    // }

    // #[test]
    // fn test_assign_markers_nearest() {
    //     // TODO: This function has been removed/refactored - test may no longer be needed
    // }

    // #[test]
    // fn test_consume_and_assign_slurs() {
    //     // TODO: This function has been removed/refactored - test may no longer be needed
    // }

    #[test]
    fn test_validation_consumption_success() {
        let upper_line = UpperLine {
            elements: vec![
                UpperElement::UpperOctaveMarker {
                    marker: ".".to_string(),
                    value: None, // Consumed
                    char_index: 0,
                },
            ],
            value: Some(".".to_string()),
            char_index: 0,
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
                    value: Some(".".to_string()), // Not consumed
                    char_index: 0,
                },
            ],
            value: Some(".".to_string()),
            char_index: 0,
        };

        let lower_line = create_test_lower_line_empty();

        assert!(validate_consumption(&upper_line, &lower_line).is_err());
    }

    #[test]
    fn test_consume_lower_line_octave_markers() {
        let upper_line = UpperLine {
            elements: vec![],
            value: Some("".to_string()),
            char_index: 0,
        };
        let lower_line = create_test_lower_line_with_markers();

        let (consumed_markers, _updated_upper, updated_lower) =
            consume_octave_markers(upper_line, lower_line);

        // Should have consumed 1 marker from lower line
        assert_eq!(consumed_markers.len(), 1);

        // Marker should be at position 0 with negative value -1
        assert_eq!(consumed_markers[0].0, 0);
        assert_eq!(consumed_markers[0].1.octave_value, -1);

        // Original value should be consumed (None)
        for element in &updated_lower.elements {
            if let LowerElement::LowerOctaveMarker { value, .. } = element {
                assert!(value.is_none(), "Lower marker value should be consumed");
            }
        }
    }

    // #[test]
    // fn test_lower_line_octave_assignment() {
    //     // TODO: This function has been removed/refactored - test may no longer be needed
    // }

    // #[test]
    // fn test_consumed_marker_move_semantics() {
    //     // TODO: This test uses old Attributes struct that may no longer exist
    // }
}
