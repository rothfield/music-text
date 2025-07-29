// src/parser/mod.rs
// Extracted parser functionality from main.rs - no logic changes

use std::collections::{HashMap, HashSet};
use crate::models::{LineInfo, Token, Node};
use crate::pitch::{lookup_pitch, guess_notation};

pub fn flatten_spatial_relationships(physical_tokens: &[Token], lines_info: &[LineInfo]) -> Vec<Node> {
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
    
    // Second pass: add all other tokens as top-level nodes
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
        let line_node = if lines_of_music.contains(&line_num) {
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
        "LINE".to_string(),
        format!("music-line-{}", line_num),
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
    let mut beats = Vec::new();
    let mut current_beat_elements = Vec::new();
    let mut beat_start_col = 0;
    
    // Sort nodes by column position
    let mut sorted_nodes = line_nodes;
    sorted_nodes.sort_by_key(|n| n.col);
    
    for node in sorted_nodes {
        if is_beat_element(&node.node_type) {
            if current_beat_elements.is_empty() {
                beat_start_col = node.col;
            }
            current_beat_elements.push(node.clone());
        } else if is_beat_separator(&node.node_type) {
            // End current beat if we have elements
            if !current_beat_elements.is_empty() {
                let beat = create_beat_node(current_beat_elements, beat_start_col);
                beats.push(beat);
                current_beat_elements = Vec::new();
            }
            // Add the separator as a regular node
            beats.push(node.clone());
        } else {
            // For other node types, end current beat and add the node
            if !current_beat_elements.is_empty() {
                let beat = create_beat_node(current_beat_elements, beat_start_col);
                beats.push(beat);
                current_beat_elements = Vec::new();
            }
            beats.push(node.clone());
        }
    }
    
    // Handle any remaining beat elements
    if !current_beat_elements.is_empty() {
        let beat = create_beat_node(current_beat_elements, beat_start_col);
        beats.push(beat);
    }
    
    beats
}

fn is_beat_element(node_type: &str) -> bool {
    matches!(node_type, "PITCH")
}

fn is_beat_separator(node_type: &str) -> bool {
    matches!(node_type, "BARLINE" | "WHITESPACE")
}

fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
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