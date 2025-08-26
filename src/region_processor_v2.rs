// Region Processor V2 Module
// Processes underscore regions for slurs and beat brackets using ParsedElement
// Creates actual SlurStart/SlurEnd elements instead of attributes

use std::collections::HashMap;
use crate::models::Token;
use crate::models_v2::{ParsedElement, Position};

pub fn apply_slurs_and_regions_to_elements(elements: &mut Vec<ParsedElement>, tokens: &[Token]) {
    let mut tokens_by_line: HashMap<usize, Vec<&Token>> = HashMap::new();
    
    // Group tokens by line
    for token in tokens {
        tokens_by_line.entry(token.line).or_default().push(token);
    }
    
    // Find underscore sequences on each line
    let mut slur_regions: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    
    for (line_num, line_tokens) in &tokens_by_line {
        let slur_tokens: Vec<_> = line_tokens.iter()
            .filter(|t| t.token_type == "SLUR" || (t.token_type == "SYMBOLS" && t.value == "_"))
            .collect();
            
        if !slur_tokens.is_empty() {
            // Handle SLUR tokens (which contain multiple underscores)
            for token in &slur_tokens {
                if token.token_type == "SLUR" {
                    let start_col = token.col;
                    let end_col = token.col + token.value.len() - 1;
                    slur_regions.entry(*line_num).or_default().push((start_col, end_col));
                }
            }
            
            // Legacy logic for individual underscore tokens
            let underscore_tokens: Vec<_> = slur_tokens.iter()
                .filter(|t| t.token_type == "SYMBOLS" && t.value == "_")
                .collect();
            
            if !underscore_tokens.is_empty() {
                let mut current_start = None;
                let mut current_end = None;
                
                for token in underscore_tokens {
                    match current_start {
                        None => {
                            current_start = Some(token.col);
                            current_end = Some(token.col);
                        }
                        Some(start) => {
                            if token.col == current_end.unwrap() + 1 {
                                current_end = Some(token.col);
                            } else {
                                if current_end.unwrap() > start {
                                    slur_regions.entry(*line_num).or_default().push((start, current_end.unwrap()));
                                }
                                current_start = Some(token.col);
                                current_end = Some(token.col);
                            }
                        }
                    }
                }
                
                if let (Some(start), Some(end)) = (current_start, current_end) {
                    if end > start {
                        slur_regions.entry(*line_num).or_default().push((start, end));
                    }
                }
            }
        }
    }
    
    // Apply slur markers - this will insert SlurStart/SlurEnd elements
    apply_slur_markers_to_elements(elements, &slur_regions);
}

fn apply_slur_markers_to_elements(elements: &mut Vec<ParsedElement>, slur_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    // Collect all note positions first
    let mut note_positions: Vec<(usize, usize, usize)> = Vec::new(); // (index, row, col)
    
    for (i, element) in elements.iter().enumerate() {
        if let ParsedElement::Note { position, .. } = element {
            note_positions.push((i, position.row, position.col));
        }
    }
    
    // Find slur start and end positions
    let mut slur_insertions: Vec<(usize, ParsedElement)> = Vec::new(); // (insert_at_index, element)
    
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
                
                // Add SlurStart before the first note
                if let Some((first_idx, first_row, first_col)) = notes_in_region.first() {
                    let slur_start = ParsedElement::SlurStart {
                        position: Position::new(*first_row, *first_col),
                    };
                    slur_insertions.push((*first_idx, slur_start));
                }
                
                // Add SlurEnd after the last note
                if let Some((last_idx, last_row, last_col)) = notes_in_region.last() {
                    let slur_end = ParsedElement::SlurEnd {
                        position: Position::new(*last_row, last_col + 1),
                    };
                    slur_insertions.push((last_idx + 1, slur_end));
                }
            }
        }
    }
    
    // Sort insertions by index (descending) to avoid index shifting issues
    slur_insertions.sort_by_key(|(idx, _)| std::cmp::Reverse(*idx));
    
    // Insert the slur markers
    for (insert_idx, slur_element) in slur_insertions {
        if insert_idx <= elements.len() {
            elements.insert(insert_idx, slur_element);
        }
    }
}