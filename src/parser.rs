// src/parser/mod.rs
// Extracted parser functionality from main.rs - no logic changes

use std::collections::{HashMap, HashSet};
use crate::models::{LineInfo, Token, Node};
use crate::pitch::{lookup_pitch, guess_notation};

fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
}

pub fn find_musical_lines_by_packed_pitches(tokens: &[Token]) -> Vec<usize> {
    let mut musical_lines = Vec::new();
    let mut tokens_by_line: HashMap<usize, Vec<&Token>> = HashMap::new();
    
    // Group tokens by line
    for token in tokens {
        if token.token_type == "PITCH" {
            tokens_by_line.entry(token.line).or_default().push(token);
        }
    }
    
    // Check each line for 3+ packed pitches from same notation system
    for (line_num, line_tokens) in tokens_by_line {
        if has_packed_musical_sequence(&line_tokens) {
            musical_lines.push(line_num);
        }
    }
    
    musical_lines.sort();
    musical_lines.dedup();
    musical_lines
}

fn has_packed_musical_sequence(tokens: &[&Token]) -> bool {
    if tokens.len() < 3 {
        return false;
    }
    
    // Sort tokens by column position
    let mut sorted_tokens: Vec<&Token> = tokens.iter().cloned().collect();
    sorted_tokens.sort_by_key(|t| t.col);
    
    // Find sequences of consecutive columns
    let mut sequences = Vec::new();
    let mut current_sequence = Vec::new();
    
    for (i, token) in sorted_tokens.iter().enumerate() {
        if i == 0 || token.col == sorted_tokens[i-1].col + 1 {
            // Consecutive or first token
            current_sequence.push(*token);
        } else {
            // Gap found - save current sequence and start new one
            if current_sequence.len() >= 3 {
                sequences.push(current_sequence.clone());
            }
            current_sequence = vec![*token];
        }
    }
    
    // Don't forget the last sequence
    if current_sequence.len() >= 3 {
        sequences.push(current_sequence);
    }
    
    // Check if any sequence has 3+ pitches from same notation system
    for sequence in sequences {
        if sequence.len() >= 3 && is_same_notation_system(&sequence) {
            return true;
        }
    }
    
    false
}

fn is_same_notation_system(tokens: &[&Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    
    // Collect all pitch values (excluding dashes)
    let pitch_values: Vec<&str> = tokens.iter()
        .map(|t| t.value.as_str())
        .filter(|&v| !is_dash(v))
        .collect();
    
    if pitch_values.is_empty() {
        return false; // All dashes - not a valid musical sequence
    }
    
    // Guess notation from the pitch values
    let notation = guess_notation(&pitch_values);
    
    // Check if all non-dash pitches belong to the same notation system
    for &pitch_value in &pitch_values {
        if lookup_pitch(pitch_value, notation).is_none() {
            return false; // This pitch doesn't belong to the detected notation system
        }
    }
    
    true
}

// ============================================================================
// PHASE 1: SPATIAL ANALYSIS 
// Attaches floating elements (ornaments, octave markers) to their anchor pitches
// ============================================================================

pub fn attach_floating_elements(physical_tokens: &[Token], lines_info: &[LineInfo]) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut consumed_coords = HashSet::new();
    let token_map: HashMap<(usize, usize), &Token> =
        physical_tokens.iter().map(|t| ((t.line, t.col), t)).collect();

    // Identify main lines: lines with barlines OR lines with pitch tokens
    let mut main_lines: HashSet<usize> = HashSet::new();
    
    // Add lines with barlines
    for token in physical_tokens.iter() {
        if token.token_type == "BARLINE" {
            main_lines.insert(token.line);
        }
    }
    
    // If no barlines found, add lines with pitch tokens
    if main_lines.is_empty() {
        for token in physical_tokens.iter() {
            if token.token_type == "PITCH" && !is_dash(&token.value) {
                main_lines.insert(token.line);
            }
        }
    }

    let mut boundaries: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut sorted_main_lines: Vec<_> = main_lines.iter().cloned().collect();
    sorted_main_lines.sort();

    for line_num in &sorted_main_lines {
        let upper_bound = (1..*line_num).rev()
            .find(|&l| main_lines.contains(&l))
            .map_or(1, |l| l + 1);
        let lower_bound = (*line_num + 1..=lines_info.len())
            .find(|&l| main_lines.contains(&l))
            .map_or(lines_info.len(), |l| l - 1);
        boundaries.insert(*line_num, (upper_bound, lower_bound));
    }

    // Detect notation type by collecting all pitch symbols
    let pitch_symbols: Vec<&str> = physical_tokens
        .iter()
        .filter(|t| t.token_type == "PITCH" && !is_dash(&t.value))
        .map(|t| t.value.as_str())
        .collect();
    let notation = guess_notation(&pitch_symbols);

    // First pass: create pitch-anchored groups
    for token in physical_tokens.iter().filter(|t| main_lines.contains(&t.line)) {
        if token.token_type == "PITCH" && !consumed_coords.contains(&(token.line, token.col)) && !is_dash(&token.value) {
            consumed_coords.insert((token.line, token.col));
            let (upper_bound, lower_bound) = boundaries.get(&token.line).cloned().unwrap_or((0, usize::MAX));
            
            let mut child_nodes = Vec::new();

            for line_idx in upper_bound..=lower_bound {
                if line_idx == token.line { continue; }
                if let Some(t) = token_map.get(&(line_idx, token.col)) {
                    if !consumed_coords.contains(&(t.line, t.col)) {
                        let token_type = t.token_type.as_str();
                        
                        if token_type == "SYMBOLS" && (t.value == "." || t.value == "'" || t.value == ":") {
                            child_nodes.push(Node::new(
                                "OCTAVE_MARKER".to_string(),
                                t.value.clone(),
                                t.line,
                                t.col,
                            ));
                            consumed_coords.insert((t.line, t.col));
                        } else if token_type == "WORD" || token_type == "METADATA" || token_type == "SYMBOLS" {
                             child_nodes.push(Node::new(
                                t.token_type.clone(),
                                t.value.clone(),
                                t.line,
                                t.col,
                            ));
                            consumed_coords.insert((t.line, t.col));
                        }
                    }
                }
            }
            
            let (pitch_code, octave) = if token.token_type == "PITCH" && !is_dash(&token.value) {
                let pitch_code = lookup_pitch(&token.value, notation);
                
                // Look for octave markers directly above or below this pitch
                let mut octave = 0i8; // default to middle octave
                
                // Check row above (upper octave markers)
                if let Some(above_token) = physical_tokens.iter().find(|t| 
                    t.line == token.line.saturating_sub(1) && 
                    t.col == token.col && 
                    t.token_type == "SYMBOLS" &&
                    (t.value == "." || t.value == "'" || t.value == ":")
                ) {
                    octave += match above_token.value.as_str() {
                        "." => 1,  // dot above means upper octave
                        "'" | ":" => 1, // apostrophe or colon above means upper octave
                        _ => 0,
                    };
                }
                
                // Check row below (lower octave markers)  
                if let Some(below_token) = physical_tokens.iter().find(|t|
                    t.line == token.line + 1 && 
                    t.col == token.col && 
                    t.token_type == "SYMBOLS" &&
                    (t.value == "." || t.value == "'" || t.value == ":")
                ) {
                    octave += match below_token.value.as_str() {
                        "." => -1, // dot below means lower octave
                        "'" | ":" => -1, // apostrophe or colon below means lower octave
                        _ => 0,
                    };
                }
                
                // Look for ornament sequences above this pitch (similar to octave markers)
                let mut ornament_children = Vec::new();  
                let (upper_bound, _) = boundaries.get(&token.line).cloned().unwrap_or((0, usize::MAX));
                
                // Search for pitch sequences in lines above this music line that align with this pitch
                for check_line in upper_bound..token.line {
                    let line_pitches: Vec<&Token> = physical_tokens.iter()
                        .filter(|t| t.line == check_line && 
                                    t.token_type == "PITCH" && 
                                    !is_dash(&t.value))
                        .collect();
                    
                    if !line_pitches.is_empty() {
                        // Check if ornament sequence starts at or near the target pitch column
                        let ornament_start_col = line_pitches.first().map(|t| t.col).unwrap_or(usize::MAX);
                        
                        // If ornament sequence starts at the same column as target pitch
                        if ornament_start_col == token.col {
                            // Convert each ornament pitch to a child node
                            for pitch_token in line_pitches {
                                let ornament_pitch_code = lookup_pitch(&pitch_token.value, notation);
                                let mut ornament_node = Node::new(
                                    "ORNAMENT".to_string(),
                                    pitch_token.value.clone(),
                                    pitch_token.line,
                                    pitch_token.col,
                                );
                                ornament_node.pitch_code = ornament_pitch_code;
                                ornament_node.octave = Some(0); // Default to middle octave for ornaments
                                ornament_children.push(ornament_node);
                                consumed_coords.insert((pitch_token.line, pitch_token.col));
                            }
                            break; // Only take the first matching ornament sequence
                        }
                    }
                }
                
                // Add ornament children to the child_nodes
                child_nodes.extend(ornament_children);
                
                (pitch_code, Some(octave))
            } else {
                (None, None)
            };
            
            let mut node = Node::with_children(
                token.token_type.clone(),
                token.value.clone(),
                token.line,
                token.col,
                child_nodes,
            );
            node.pitch_code = pitch_code;
            node.octave = octave;
            nodes.push(node);
        }
    }
    
    // Second pass: add all other tokens as top-level nodes (including WHITESPACE and BARLINE)
    for token in physical_tokens {
        if !consumed_coords.contains(&(token.line, token.col)) {
            let (pitch_code, octave) = if token.token_type == "PITCH" && !is_dash(&token.value) {
                let pitch_code = lookup_pitch(&token.value, notation);
                
                // Look for octave markers directly above or below this pitch
                let mut octave = 0i8; // default to middle octave
                
                // Check row above (upper octave markers)
                if let Some(above_token) = physical_tokens.iter().find(|t| 
                    t.line == token.line.saturating_sub(1) && 
                    t.col == token.col && 
                    t.token_type == "SYMBOLS" &&
                    (t.value == "." || t.value == "'" || t.value == ":")
                ) {
                    octave += match above_token.value.as_str() {
                        "." => 1,  // dot above means upper octave
                        "'" | ":" => 1, // apostrophe or colon above means upper octave
                        _ => 0,
                    };
                }
                
                // Check row below (lower octave markers)  
                if let Some(below_token) = physical_tokens.iter().find(|t|
                    t.line == token.line + 1 && 
                    t.col == token.col && 
                    t.token_type == "SYMBOLS" &&
                    (t.value == "." || t.value == "'" || t.value == ":")
                ) {
                    octave += match below_token.value.as_str() {
                        "." => -1, // dot below means lower octave
                        "'" | ":" => -1, // apostrophe or colon below means lower octave
                        _ => 0,
                    };
                }
                
                (pitch_code, Some(octave))
            } else {
                (None, None)
            };
            
            let mut node = Node::new(
                token.token_type.clone(),
                token.value.clone(),
                token.line,
                token.col,
            );
            node.pitch_code = pitch_code;
            node.octave = octave;
            nodes.push(node);
        }
    }

    nodes.sort_by(|a, b| a.row.cmp(&b.row).then_with(|| a.col.cmp(&b.col)));

    nodes
}

// ============================================================================  
// PHASE 2: MUSICAL STRUCTURING
// Groups hierarchical nodes into musical constructs (lines, beats)
// ============================================================================

pub fn group_nodes_into_lines_and_beats(nodes: &[Node], lines_of_music: &Vec<usize>) -> Vec<Node> {
    let mut result = Vec::new();
    let mut nodes_by_line: HashMap<usize, Vec<&Node>> = HashMap::new();
    
    // Group nodes by line
    for node in nodes {
        nodes_by_line.entry(node.row).or_default().push(node);
    }
    
    // Process each line
    let mut sorted_lines: Vec<_> = nodes_by_line.into_iter().collect();
    sorted_lines.sort_by_key(|(line_num, _)| *line_num);
    
    for (line_num, line_nodes) in sorted_lines {
        // A line is musical if it's in the pre-identified list OR if it contains any PITCH nodes.
        let is_musical = lines_of_music.contains(&line_num) || line_nodes.iter().any(|n| n.node_type == "PITCH");
        
        let line_node = if is_musical {
            // This is a line of music - create LINE node with BEAT children
            create_music_line_node(line_num, line_nodes)
        } else {
            // Non-music line - create LINE node with direct children
            create_regular_line_node(line_num, line_nodes)
        };
        result.push(line_node);
    }
    
    result
}

fn create_music_line_node(line_num: usize, line_nodes: Vec<&Node>) -> Node {
    let beats_and_separators = group_line_into_beats(line_nodes);
    let start_col = beats_and_separators.first().map(|n| n.col).unwrap_or(0);
    
    Node::with_children(
        "MUSICAL_LINE".to_string(),
        format!("line-{}", line_num),
        line_num,
        start_col,
        beats_and_separators,
    )
}

fn create_regular_line_node(line_num: usize, line_nodes: Vec<&Node>) -> Node {
    let mut sorted_nodes = line_nodes;
    sorted_nodes.sort_by_key(|n| n.col);
    let start_col = sorted_nodes.first().map(|n| n.col).unwrap_or(0);
    
    Node::with_children(
        "LINE".to_string(),
        format!("line-{}", line_num),
        line_num,
        start_col,
        sorted_nodes.into_iter().cloned().collect(),
    )
}

fn group_line_into_beats(line_nodes: Vec<&Node>) -> Vec<Node> {
    let mut result = Vec::new();
    let mut accumulator = Vec::new();

    // Sort nodes by column position to ensure correct order
    let mut sorted_nodes = line_nodes;
    sorted_nodes.sort_by_key(|n| n.col);
    
    for node in sorted_nodes {
        if is_beat_element(&node.node_type) {
            // If it's a pitch, add it to the current beat accumulator
            accumulator.push(node.clone());
        } else {
            // If it's not a pitch (e.g., whitespace, barline), the beat has ended.
            
            // 1. If the accumulator has pitches, create a BEAT node.
            if !accumulator.is_empty() {
                let start_col = accumulator.first().unwrap().col;
                let beat_node = create_beat_node(accumulator, start_col);
                result.push(beat_node);
                accumulator = Vec::new(); // Clear the accumulator for the next beat
            }
            
            // 2. Add the separator itself (whitespace, barline, etc.) to the result.
            result.push(node.clone());
        }
    }

    // After the loop, if there are any remaining pitches in the accumulator, create one final beat.
    if !accumulator.is_empty() {
        let start_col = accumulator.first().unwrap().col;
        let beat_node = create_beat_node(accumulator, start_col);
        result.push(beat_node);
    }

    result
}

fn is_beat_element(node_type: &str) -> bool {
    matches!(node_type, "PITCH")
}


fn create_beat_node(mut elements: Vec<Node>, start_col: usize) -> Node {
    // Count trailing dashes for each pitch and mark them as consumed
    let mut i = 0;
    while i < elements.len() {
        if elements[i].node_type == "PITCH" && !is_dash(&elements[i].value) {
            // This is a non-dash pitch - count its trailing dashes
            let mut subdivisions = 1; // The pitch itself
            let mut j = i + 1;
            
            // Count consecutive dashes following this pitch
            while j < elements.len() && 
                  elements[j].node_type == "PITCH" && 
                  is_dash(&elements[j].value) {
                subdivisions += 1;
                elements[j].dash_consumed = true; // Mark dash as consumed
                j += 1;
            }
            
            // Set the pitch's subdivision count
            elements[i].divisions = subdivisions;
        } else if elements[i].node_type == "PITCH" && !elements[i].dash_consumed {
            // This is an unconsumed dash, treat as rest with 1 subdivision
            elements[i].divisions = 1;
        }
        i += 1;
    }
    
    let total_subdivisions = elements.iter()
        .filter(|n| n.node_type == "PITCH")
        .count();
        
    let row = elements.first().map(|n| n.row).unwrap_or(0);
    
    let mut node = Node::with_children(
        "BEAT".to_string(),
        format!("beat-{}", total_subdivisions),
        row,
        start_col,
        elements,
    );
    node.divisions = total_subdivisions;
    node
}
