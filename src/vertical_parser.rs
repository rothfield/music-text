// Vertical Parser (formerly Region Processor)
// Processes vertical relationships like slurs and ornaments.

use std::collections::HashMap;
use crate::models::Token;
use crate::parsed_models::{ParsedElement, ParsedChild, OrnamentType, SlurRole};

pub fn apply_slurs_and_regions_to_elements(elements: &mut Vec<ParsedElement>, tokens: &[Token]) {
    // Apply ornaments first, as they are simpler attachments and can be removed from the stream.
    apply_ornaments_to_elements(elements);
    
    // Apply syllables to notes using spatial snapping (same algorithm as ornaments)
    apply_syllables_to_elements(elements);
    
    // Convert tala markers (+ and numbers) for all notation systems
    convert_numbers_above_pitches_to_talas(elements);
    
    // Assign tala markers sequentially to barlines
    assign_talas(elements);

    let mut tokens_by_line: HashMap<usize, Vec<&Token>> = HashMap::new();
    
    // Group tokens by line
    for token in tokens {
        tokens_by_line.entry(token.line).or_default().push(token);
    }
    
    // Find underscore sequences on each line for slurs
    let mut slur_regions: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    
    for (line_num, line_tokens) in &tokens_by_line {
        let slur_tokens: Vec<_> = line_tokens.iter()
            .filter(|t| t.token_type == "SLUR" || (t.token_type == "SYMBOLS" && t.value == "_"))
            .collect();
            
        if !slur_tokens.is_empty() {
            for token in &slur_tokens {
                if token.token_type == "SLUR" {
                    let start_col = token.col;
                    let end_col = token.col + token.value.len() - 1;
                    slur_regions.entry(*line_num).or_default().push((start_col, end_col));
                }
            }
        }
    }
    
    // Apply slur roles directly to the notes based on the underscore regions
    apply_slur_roles_to_elements(elements, &slur_regions);
}

/// Finds ornament symbols (like '~') and attaches them as children to the nearest note below.
fn apply_ornaments_to_elements(elements: &mut Vec<ParsedElement>) {
    let mut ornament_indices = Vec::new();
    let mut note_indices = Vec::new();

    // First, get the indices of all potential ornaments and notes.
    for (i, element) in elements.iter().enumerate() {
        if let ParsedElement::Symbol { value, .. } = element {
            if value == "~" { // This can be expanded for other ornaments
                ornament_indices.push(i);
            }
        } else if let ParsedElement::Note { .. } = element {
            note_indices.push(i);
        }
    }

    let mut consumed_ornament_indices = std::collections::HashSet::new();

    // For each note, find the best ornament symbol above it.
    for &note_idx in &note_indices {
        let note_pos = elements[note_idx].position();
        let mut best_ornament_idx: Option<usize> = None;
        let mut min_dist = isize::MAX;

        for &ornament_idx in &ornament_indices {
            if consumed_ornament_indices.contains(&ornament_idx) {
                continue;
            }

            let ornament_pos = elements[ornament_idx].position();

            // Check if ornament is on a line above the note and horizontally close.
            if ornament_pos.row < note_pos.row {
                let dist = (note_pos.col as isize - ornament_pos.col as isize).abs();
                if dist < min_dist {
                    min_dist = dist;
                    best_ornament_idx = Some(ornament_idx);
                }
            }
        }

        // If a suitable ornament was found, attach it to the note.
        if let Some(ornament_idx) = best_ornament_idx {
            // This check ensures we don't attach an ornament that is very far away horizontally.
            if min_dist < 5 {
                 if let ParsedElement::Note { children, .. } = &mut elements[note_idx] {
                    children.push(ParsedChild::Ornament {
                        kind: OrnamentType::Mordent,
                        distance: -1, // Indicates it was above the note.
                    });
                    consumed_ornament_indices.insert(ornament_idx);
                }
            }
        }
    }

    // Remove the consumed ornament symbols from the main element list.
    let mut i = 0;
    elements.retain(|_| {
        let keep = !consumed_ornament_indices.contains(&i);
        i += 1;
        keep
    });
}


fn apply_slur_roles_to_elements(elements: &mut Vec<ParsedElement>, slur_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    // Collect all note positions first
    let mut note_positions: Vec<(usize, usize, usize)> = Vec::new(); // (index, row, col)
    
    for (i, element) in elements.iter().enumerate() {
        if let ParsedElement::Note { position, .. } = element {
            note_positions.push((i, position.row, position.col));
        }
    }
    
    // Assign slur roles to notes in regions
    for (slur_line, regions) in slur_regions {
        for (start_col, end_col) in regions {
            // Find notes that should have slur markers
            let mut notes_in_region: Vec<(usize, usize, usize)> = Vec::new(); // (index, row, col)
            
            for &(idx, note_row, note_col) in &note_positions {
                // Look for notes on lines below the slur region (check up to 3 lines below)
                for distance in 1..=3 {
                    if note_row == slur_line + distance {
                        // Check if note falls within or near the slur region
                        if note_col >= *start_col && note_col <= *end_col {
                            notes_in_region.push((idx, note_row, note_col));
                        } else if note_col <= *start_col && (*start_col - note_col) <= 3 {
                            // Note slightly before region (for slur start)
                            notes_in_region.push((idx, note_row, note_col));
                        } else if note_col > *end_col && (note_col - *end_col) <= 3 {
                            // Note slightly after region (for slur end)
                            notes_in_region.push((idx, note_row, note_col));
                        }
                        break;
                    }
                }
            }
            
            if !notes_in_region.is_empty() {
                // Sort by column position
                notes_in_region.sort_by_key(|(_, _, col)| *col);
                
                // Assign slur roles directly to the notes
                for (i, (note_idx, _, _)) in notes_in_region.iter().enumerate() {
                    if let ParsedElement::Note { slur, .. } = &mut elements[*note_idx] {
                        *slur = Some(match (i, notes_in_region.len()) {
                            (0, 1) => SlurRole::StartEnd,  // Single note slur
                            (0, _) => SlurRole::Start,     // First note
                            (n, len) if n == len - 1 => SlurRole::End, // Last note  
                            _ => SlurRole::Middle,         // Middle notes
                        });
                    }
                }
            }
        }
    }
}

/// Converts numbers above pitches to tala markers (only for Number notation system)
fn convert_numbers_above_pitches_to_talas(elements: &mut Vec<ParsedElement>) {
    let mut symbol_indices = Vec::new();
    let mut pitch_indices = Vec::new();

    // Collect indices of symbols (numbers) and pitches
    for (i, element) in elements.iter().enumerate() {
        match element {
            ParsedElement::Symbol { value, .. } => {
                // Consider tala markers: + and numbers 0-6 
                if value.len() == 1 && matches!(value.chars().next(), Some('+' | '0'..='6')) {
                    symbol_indices.push(i);
                }
            }
            ParsedElement::Note { .. } => {
                pitch_indices.push(i);
            }
            _ => {}
        }
    }

    let mut to_convert: Vec<usize> = Vec::new();

    // Check each symbol to see if it's above a pitch
    for &symbol_idx in &symbol_indices {
        let symbol_pos = elements[symbol_idx].position();
        
        // Look for pitches on lines below this symbol (within 3 lines)
        for &pitch_idx in &pitch_indices {
            let pitch_pos = elements[pitch_idx].position();
            
            // Check if symbol is above the pitch (higher line, lower row number)
            if symbol_pos.row < pitch_pos.row {
                let vertical_distance = pitch_pos.row - symbol_pos.row;
                let horizontal_distance = (symbol_pos.col as isize - pitch_pos.col as isize).abs();
                
                // If symbol is reasonably close vertically (1-3 lines) and horizontally (within 10 chars)
                if vertical_distance >= 1 && vertical_distance <= 3 && horizontal_distance <= 10 {
                    to_convert.push(symbol_idx);
                    break; // Found a pitch below, convert this symbol
                }
            }
        }
    }

    // Convert symbols to tala markers
    for &idx in &to_convert {
        if let ParsedElement::Symbol { value, position } = &elements[idx] {
            let tala_number = match value.chars().next().unwrap() {
                '+' => 255, // Special marker for + (use max u8 value as special case)
                ch => ch.to_digit(10).unwrap() as u8,
            };
            elements[idx] = ParsedElement::Tala { 
                number: tala_number, 
                position: position.clone()
            };
        }
    }
}



/// Finds word elements (syllables) and attaches them as children to notes using simple left-to-right order matching.
/// Manual positioning: matches syllables to notes in order without complex spatial calculations.
fn apply_syllables_to_elements(elements: &mut Vec<ParsedElement>) {
    let mut syllables: Vec<(usize, String)> = Vec::new();
    let mut notes: Vec<usize> = Vec::new();

    // Collect syllables and notes in document order
    for (i, element) in elements.iter().enumerate() {
        if let ParsedElement::Word { text, .. } = element {
            syllables.push((i, text.clone()));
        } else if let ParsedElement::Note { .. } = element {
            notes.push(i);
        }
    }

    let mut consumed_syllable_indices = std::collections::HashSet::new();

    // Simple left-to-right matching: assign syllables to notes in order
    for (syllable_index, (syllable_idx, syllable_text)) in syllables.iter().enumerate() {
        if consumed_syllable_indices.contains(syllable_idx) {
            continue;
        }

        // Match syllable to note at same index, or closest available note
        if let Some(&note_idx) = notes.get(syllable_index) {
            if let ParsedElement::Note { children, .. } = &mut elements[note_idx] {
                children.push(ParsedChild::Syllable {
                    text: syllable_text.clone(),
                    distance: 1, // Indicates it was below the note.
                });
                consumed_syllable_indices.insert(*syllable_idx);
            }
        }
    }

    // Remove the consumed syllable elements from the main element list.
    let mut i = 0;
    while i < elements.len() {
        if consumed_syllable_indices.contains(&i) {
            elements.remove(i);
        } else {
            i += 1;
        }
    }
}

/// Assigns tala markers sequentially to barlines.
fn assign_talas(elements: &mut Vec<ParsedElement>) {
    let mut tala_indices = Vec::new();
    let mut barline_indices = Vec::new();

    // Collect all tala markers and barlines
    for (i, element) in elements.iter().enumerate() {
        match element {
            ParsedElement::Tala { .. } => {
                tala_indices.push(i);
            }
            ParsedElement::Barline { .. } => {
                barline_indices.push(i);
            }
            _ => {}
        }
    }

    // Sort talas by position (row first, then column)
    tala_indices.sort_by_key(|&idx| {
        let pos = elements[idx].position();
        (pos.row, pos.col)
    });

    // Sort barlines by position (row first, then column)  
    barline_indices.sort_by_key(|&idx| {
        let pos = elements[idx].position();
        (pos.row, pos.col)
    });

    let mut consumed_tala_indices = std::collections::HashSet::new();

    // Assign talas sequentially to barlines
    for (tala_idx, &barline_idx) in tala_indices.iter().zip(barline_indices.iter()) {
        // Get the tala number
        if let ParsedElement::Tala { number, .. } = &elements[*tala_idx] {
            let tala_number = *number;
            eprintln!("DEBUG: Assigning tala {} to barline at index {}", tala_number, barline_idx);
            
            // Set the tala field on the barline
            if let ParsedElement::Barline { tala, .. } = &mut elements[barline_idx] {
                *tala = Some(tala_number);
                eprintln!("DEBUG: Successfully set tala {} on barline", tala_number);
            }
            consumed_tala_indices.insert(*tala_idx);
        }
    }

    // Mark any leftover talas as consumed (they will be discarded)
    for &tala_idx in &tala_indices {
        consumed_tala_indices.insert(tala_idx);
    }

    // Remove all tala elements from the main element list
    let mut i = 0;
    elements.retain(|_| {
        let keep = !consumed_tala_indices.contains(&i);
        i += 1;
        keep
    });
}
