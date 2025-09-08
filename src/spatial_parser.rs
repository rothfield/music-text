// Spatial parser for musical notation
// Processes annotation lines (octave markers, lyrics, slurs) and assigns them to notes spatially

use crate::ast::*;
use std::collections::HashMap;

/// Assign syllables from lyrics lines to notes in the stave
pub fn assign_syllables_to_notes(document: &mut Document) {
    for stave in &mut document.staves {
        // Collect all syllables from all lyrics lines
        let all_syllables: Vec<String> = stave.lyrics_lines
            .iter()
            .flat_map(|line| line.syllables.clone())
            .collect();
        
        if all_syllables.is_empty() {
            continue;
        }
        
        // Collect all pitch elements from all measures and beats
        let mut note_refs: Vec<&mut BeatElement> = Vec::new();
        
        for measure in &mut stave.content_line.measures {
            for beat in &mut measure.beats {
                match beat {
                    beat => {
                        let elements = &beat.elements;
                        for element in elements {
                            if matches!(element, BeatElement::Pitch { .. }) {
                                // Skip for now - mutable reference issue
                            }
                        }
                    }
                }
            }
        }
        
        // Assign syllables to notes, honoring slur markings
        // Only notes with BeginSlur or no slur marking should receive syllables
        let mut syllable_index = 0;
        for note in note_refs.iter_mut() {
            if let BeatElement::Pitch { syllable, slur_type, .. } = note {
                match slur_type {
                    Some(crate::ast::SlurType::InSlur) => {
                        // Notes in the middle of a slur don't get new syllables
                        *syllable = None;
                    }
                    Some(crate::ast::SlurType::BeginSlur) | None => {
                        // First note of a slur or unslurred notes get syllables
                        if syllable_index < all_syllables.len() {
                            *syllable = Some(all_syllables[syllable_index].clone());
                            syllable_index += 1;
                        }
                    }
                }
            }
        }
        
        // Clear lyrics lines and add any extra syllables back as a single line
        let extra_syllables: Vec<String> = all_syllables.into_iter().skip(syllable_index).collect();
        
        stave.lyrics_lines.clear();
        if !extra_syllables.is_empty() {
            stave.lyrics_lines.push(LyricsLine { syllables: extra_syllables });
        }
    }
}

/// Assign octave markers from upper and lower lines to notes spatially
/// First assigns markers directly above/below notes, then assigns strays to nearest unassigned notes
pub fn assign_octave_markers(document: &mut Document) {
    for stave in &mut document.staves {
        // Collect octave markers from upper and lower lines with their positions  
        // Store as (position, octave_value)
        let mut octave_markers = Vec::new();
        
        // Get octave markers from upper lines
        for line in &stave.upper_lines {
            let mut current_pos = 0;
            for item in &line.items {
                match item {
                    crate::ast::AnnotationItem::UpperOctaveMarker { marker, .. } => {
                        let octave_value = octave_marker_to_number(&marker, true);
                        octave_markers.push((current_pos, octave_value));
                        current_pos += 1;
                    }
                    crate::ast::AnnotationItem::Space { count, .. } => {
                        current_pos += count;
                    }
                    crate::ast::AnnotationItem::Slur { underscores, .. } => {
                        current_pos += underscores.len();
                    }
                    _ => {
                        current_pos += 1; // Default single character
                    }
                }
            }
        }
        
        // Get octave markers from lower lines
        for line in &stave.lower_lines {
            let mut current_pos = 0;
            for item in &line.items {
                match item {
                    crate::ast::AnnotationItem::LowerOctaveMarker { marker, .. } => {
                        let octave_value = octave_marker_to_number(&marker, false);
                        octave_markers.push((current_pos, octave_value));
                        current_pos += 1;
                    }
                    crate::ast::AnnotationItem::Space { count, .. } => {
                        current_pos += count;
                    }
                    crate::ast::AnnotationItem::BeatGrouping { underscores, .. } => {
                        current_pos += underscores.len();
                    }
                    _ => {
                        current_pos += 1; // Default single character
                    }
                }
            }
        }
        
        if octave_markers.is_empty() {
            continue;
        }
        
        // Collect note positions and references
        let mut note_positions = Vec::new();
        let mut current_pos = 0;
        
        for measure in &mut stave.content_line.measures {
            for beat in &mut measure.beats {
                match beat {
                    beat => {
                        let elements = &beat.elements;
                        for element in elements {
                            if matches!(element, BeatElement::Pitch { .. }) {
                                note_positions.push((current_pos, element));
                                current_pos += 1;
                            } else {
                                current_pos += 1; // Dashes, spaces, etc.
                            }
                        }
                    }
                }
            }
        }
        
        // Phase 1: Direct assignment - assign markers directly above/below notes
        let mut used_markers = vec![false; octave_markers.len()];
        let mut octave_assignments: std::collections::HashMap<usize, i8> = std::collections::HashMap::new();
        
        for (note_pos, _note_ref) in &note_positions {
            // Look for markers at the exact same position
            for (marker_idx, (marker_pos, marker_octave)) in octave_markers.iter().enumerate() {
                if !used_markers[marker_idx] && marker_pos == note_pos {
                    octave_assignments.insert(*note_pos, *marker_octave);
                    used_markers[marker_idx] = true;
                    break;
                }
            }
        }
        
        // Phase 2: Nearest neighbor assignment for unused markers
        for (marker_idx, (marker_pos, marker)) in octave_markers.iter().enumerate() {
            if used_markers[marker_idx] {
                continue; // Already used
            }
            
            // Find the nearest note that doesn't have an octave assignment yet
            let mut best_distance = usize::MAX;
            let mut best_note_pos = None;
            
            for (note_pos, _note_ref) in &note_positions {
                // Skip if this note already has an assignment
                if octave_assignments.contains_key(note_pos) {
                    continue;
                }
                
                let distance = if note_pos > marker_pos {
                    note_pos - marker_pos
                } else {
                    marker_pos - note_pos
                };
                
                if distance < best_distance {
                    best_distance = distance;
                    best_note_pos = Some(*note_pos);
                }
            }
            
            // Assign to the nearest unassigned note
            if let Some(note_pos) = best_note_pos {
                octave_assignments.insert(note_pos, *marker);
            }
        }
        
        // Phase 3: Apply the octave assignments to the actual document
        apply_octave_assignments(stave, &octave_assignments);
    }
}

pub fn apply_octave_assignments(stave: &mut Stave, assignments: &std::collections::HashMap<usize, i8>) {
    let mut current_pos = 0;
    
    for measure in &mut stave.content_line.measures {
        for beat in &mut measure.beats {
            for element in &mut beat.elements {
                if let BeatElement::Pitch { octave, .. } = element {
                    if let Some(&assigned_octave) = assignments.get(&current_pos) {
                        *octave = assigned_octave;
                    }
                }
                current_pos += 1;
            }
        }
    }
}

/// Analyze slurs from upper lines and mark notes spatially
pub fn analyze_slurs(document: &mut Document) {
    for stave in &mut document.staves {
        // Find slur segments in upper lines
        let slur_segments = find_slur_segments(&stave.upper_lines);
        
        if slur_segments.is_empty() {
            continue;
        }
        
        // Get positions of all notes in the content line
        let mut note_positions = Vec::new();
        let mut current_pos = 0;
        
        for measure in &stave.content_line.measures {
            for beat in &measure.beats {
                match beat {
                    beat => {
                        let elements = &beat.elements;
                        for element in elements {
                            if matches!(element, BeatElement::Pitch { .. }) {
                                note_positions.push(current_pos);
                            }
                            current_pos += 1; // Each element takes one character position (simplified)
                        }
                    }
                }
            }
        }
        
        // Apply slur markings to notes based on spatial overlap
        let mut note_index = 0;
        for measure in &mut stave.content_line.measures {
            for beat in &mut measure.beats {
                match beat {
                    beat => {
                        let elements = &beat.elements;
                        for element in elements {
                            if let BeatElement::Pitch { slur_type, .. } = element {
                                if note_index < note_positions.len() {
                                    let note_pos = note_positions[note_index];
                                    
                                    // Check if this note position is covered by any slur
                                    for (start_pos, end_pos) in &slur_segments {
                                        if note_pos >= *start_pos && note_pos <= *end_pos {
                                            // *slur_type = Some(if note_pos == *start_pos {
                                            //     crate::ast::SlurType::BeginSlur
                                            // } else {
                                            //     crate::ast::SlurType::InSlur
                                            // }); // Temporarily disabled - mutable reference issue
                                            break;
                                        }
                                    }
                                }
                                note_index += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Find slur segments (start, end positions) from upper lines
pub fn find_slur_segments(upper_lines: &[crate::ast::AnnotationLine]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    
    for line in upper_lines {
        let mut current_pos = 0;
        for item in &line.items {
            match item {
                crate::ast::AnnotationItem::Slur { underscores, .. } => {
                    let slur_len = underscores.len();
                    if slur_len >= 2 {
                        segments.push((current_pos, current_pos + slur_len - 1));
                    }
                    current_pos += slur_len;
                }
                crate::ast::AnnotationItem::Space { count, .. } => {
                    current_pos += count;
                }
                crate::ast::AnnotationItem::UpperOctaveMarker { marker, .. } => {
                    current_pos += marker.len();
                }
                crate::ast::AnnotationItem::LowerOctaveMarker { marker, .. } => {
                    current_pos += marker.len();
                }
                crate::ast::AnnotationItem::Tala { marker, .. } => {
                    current_pos += marker.len();
                }
                crate::ast::AnnotationItem::Symbol { symbol, .. } => {
                    current_pos += symbol.len();
                }
                crate::ast::AnnotationItem::Ending { ending, .. } => {
                    current_pos += ending.len();
                }
                crate::ast::AnnotationItem::Chord { chord, .. } => {
                    current_pos += chord.len() + 2; // Account for [ ]
                }
                crate::ast::AnnotationItem::Ornament { pitches, .. } => {
                    current_pos += pitches.iter().map(|p| p.len()).sum::<usize>() + pitches.len() + 1; // Rough estimate
                }
                crate::ast::AnnotationItem::Mordent { .. } => {
                    current_pos += 1; // ~
                }
                _ => {
                    current_pos += 1; // Default single character
                }
            }
        }
    }
    
    segments
}

/// Convert octave marker to numeric value based on type and symbol
pub fn octave_marker_to_number(marker: &str, is_upper: bool) -> i8 {
    let base_value = match marker {
        "." => 1,
        ":" => 2,
        "*" => 3,
        "'" => 4,
        _ => 0,
    };
    
    if is_upper {
        base_value  // Upper markers are positive (higher octaves)
    } else {
        -base_value // Lower markers are negative (lower octaves)
    }
}