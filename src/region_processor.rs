// Region Processor Module
// Handles Phase 2 of spatial analysis: processing underscore regions for slurs and beat brackets
// Takes hierarchical nodes and marks them with slur_start/slur_end and beat_bracket_start/beat_bracket_end attributes

use std::collections::HashMap;
use crate::models::{Token, Node};

pub fn apply_slurs_and_regions_to_nodes(nodes: &mut [Node], tokens: &[Token]) {
    let mut tokens_by_line: HashMap<usize, Vec<&Token>> = HashMap::new();
    
    // Group tokens by line
    for token in tokens {
        tokens_by_line.entry(token.line).or_default().push(token);
    }
    
    // Find underscore sequences on each line
    // NOTE: In this notation system, slurs are indicated by underscores (___) ABOVE the main note line
    let mut slur_regions: HashMap<usize, Vec<(usize, usize)>> = HashMap::new(); // line -> (start_col, end_col)
    
    for (line_num, line_tokens) in &tokens_by_line {
        let slur_tokens: Vec<_> = line_tokens.iter()
            .filter(|t| t.token_type == "SLUR" || (t.token_type == "SYMBOLS" && t.value == "_"))
            .collect();
            
        if !slur_tokens.is_empty() {
            // Handle SLUR tokens (which contain multiple underscores) and individual underscore SYMBOLS
            for token in &slur_tokens {
                if token.token_type == "SLUR" {
                    // SLUR tokens represent a complete slur region
                    let start_col = token.col;
                    let end_col = token.col + token.value.len() - 1;
                    slur_regions.entry(*line_num).or_default().push((start_col, end_col));
                }
            }
            
            // Legacy logic for individual underscore tokens (can be removed later)
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
                            // Consecutive underscore
                            current_end = Some(token.col);
                        } else {
                            // Gap found - finish current slur region (only if it spans multiple underscores)
                            if current_end.unwrap() > start {
                                slur_regions.entry(*line_num).or_default().push((start, current_end.unwrap()));
                            }
                            current_start = Some(token.col);
                            current_end = Some(token.col);
                        }
                    }
                }
            }
            
                // Don't forget the last region - but only if it spans multiple underscores
                if let (Some(start), Some(end)) = (current_start, current_end) {
                    if end > start {  // Only create slur regions for multiple underscores
                        slur_regions.entry(*line_num).or_default().push((start, end));
                    }
                }
            }
        }
    }
    
    // Apply slur attributes from underscores
    apply_slur_attributes_recursive(nodes, &slur_regions);
    
    // Also apply beat bracket attributes using the same underscore regions but looking below pitches
    apply_beat_bracket_attributes(nodes, &slur_regions);
    
    // Note: Syllables are already handled in attach_floating_elements as SYL child nodes
}

fn apply_beat_bracket_attributes(nodes: &mut [Node], underscore_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    // Apply recursively to handle hierarchical structure from FSM
    apply_beat_bracket_attributes_recursive(nodes, underscore_regions);
}

fn apply_beat_bracket_attributes_recursive(nodes: &mut [Node], underscore_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    // For each underscore region, find all pitches that fall within it
    for (_line_num, regions) in underscore_regions {
        for (start_col, end_col) in regions {
            let mut pitch_indices: Vec<Vec<usize>> = Vec::new(); // Path to each pitch
            
            // First pass: collect paths to pitches that fall within this underscore region
            collect_pitch_paths_in_region(nodes, underscore_regions, *start_col, *end_col, &mut Vec::new(), &mut pitch_indices);
            
            // Mark the first and last pitches in the region
            if !pitch_indices.is_empty() {
                // Sort by column position of actual pitches
                pitch_indices.sort_by_key(|path| get_node_at_path(nodes, path).col);
                
                // Mark first as beat_bracket_start
                if let Some(first_path) = pitch_indices.first() {
                    if let Some(first_node) = get_node_at_path_mut(nodes, first_path) {
                        first_node.beat_bracket_start = Some(true);
                    }
                }
                
                // Mark last as beat_bracket_end
                if let Some(last_path) = pitch_indices.last() {
                    if let Some(last_node) = get_node_at_path_mut(nodes, last_path) {
                        last_node.beat_bracket_end = Some(true);
                    }
                }
            }
        }
    }
}

fn collect_pitch_paths_in_region(
    nodes: &[Node], 
    underscore_regions: &HashMap<usize, Vec<(usize, usize)>>, 
    start_col: usize, 
    end_col: usize, 
    current_path: &mut Vec<usize>, 
    pitch_indices: &mut Vec<Vec<usize>>
) {
    for (i, node) in nodes.iter().enumerate() {
        current_path.push(i);
        
        if node.node_type == "PITCH" && node.pitch_code.is_some() {
            // Look for underscore regions on lines below (check up to 3 lines below)
            for distance in 1..=3 {
                let check_line = node.row + distance;
                if let Some(check_regions) = underscore_regions.get(&check_line) {
                    for (check_start, check_end) in check_regions {
                        // Check if this note falls within this underscore region
                        if node.col >= *check_start && node.col <= *check_end &&
                           *check_start == start_col && *check_end == end_col {
                            pitch_indices.push(current_path.clone());
                            break;
                        }
                    }
                }
            }
        }
        
        // Recursively check child nodes
        collect_pitch_paths_in_region(&node.nodes, underscore_regions, start_col, end_col, current_path, pitch_indices);
        
        current_path.pop();
    }
}

fn get_node_at_path<'a>(nodes: &'a [Node], path: &[usize]) -> &'a Node {
    let mut current_node = &nodes[path[0]];
    for &index in &path[1..] {
        current_node = &current_node.nodes[index];
    }
    current_node
}

fn get_node_at_path_mut<'a>(nodes: &'a mut [Node], path: &[usize]) -> Option<&'a mut Node> {
    if path.is_empty() {
        return None;
    }
    
    let mut current_node = &mut nodes[path[0]];
    for &index in &path[1..] {
        if index >= current_node.nodes.len() {
            return None;
        }
        current_node = &mut current_node.nodes[index];
    }
    Some(current_node)
}

// NOTE: In this notation system, slurs are indicated by underscores (___) ABOVE the main note line,
// not on the same line as the notes. Underscores on the same line as notes have different meaning.
fn apply_slur_attributes_recursive(nodes: &mut [Node], slur_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    // First, collect information about all pitch nodes and their positions
    let mut pitch_positions: Vec<(usize, usize)> = Vec::new(); // (row, col)
    collect_pitch_positions(nodes, &mut pitch_positions);
    
    // Use a simpler two-pass approach without complex borrowing
    for node in &mut *nodes {
        if node.node_type == "PITCH" && node.pitch_code.is_some() {
            // Look for slur regions on lines above (check up to 3 lines above)
            for distance in 1..=3 {
                if let Some(check_line) = node.row.checked_sub(distance) {
                    if let Some(regions) = slur_regions.get(&check_line) {
                        for (start_col, end_col) in regions {
                            // For slur start: mark the first note that falls within or near the slur region start
                            if node.col >= *start_col && node.col <= *end_col {
                                // This note is within the slur region - it could be the start
                                // Check if there are any previous notes in this slur region
                                let is_first_note_in_slur = !pitch_positions.iter().any(|(_, other_col)| {
                                    *other_col >= *start_col && 
                                    *other_col < node.col &&
                                    *other_col <= *end_col
                                });
                                
                                if is_first_note_in_slur {
                                    node.slur_start = Some(true);
                                }
                                break;
                            }
                            // Also check for notes slightly before the slur region (legacy behavior)
                            else if node.col <= *start_col && 
                               (*start_col - node.col) <= 3 {  // Within reasonable distance (3 columns)
                                node.slur_start = Some(true);
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Recursively process child nodes
        apply_slur_attributes_recursive(&mut node.nodes, slur_regions);
    }
    
    // Second pass: mark the last note in each slur region with slur_end
    mark_slur_end_nodes(nodes, slur_regions);
}

// Helper function to collect pitch positions
fn collect_pitch_positions(nodes: &[Node], positions: &mut Vec<(usize, usize)>) {
    for node in nodes {
        if node.node_type == "PITCH" && node.pitch_code.is_some() {
            positions.push((node.row, node.col));
        }
        collect_pitch_positions(&node.nodes, positions);
    }
}

fn mark_slur_end_nodes(nodes: &mut [Node], slur_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    for node in &mut *nodes {
        if node.node_type == "PITCH" && node.pitch_code.is_some() {
            // Look for slur regions on lines above (check up to 3 lines above)
            for distance in 1..=3 {
                if let Some(check_line) = node.row.checked_sub(distance) {
                    if let Some(regions) = slur_regions.get(&check_line) {
                        for (start_col, end_col) in regions {
                            // For slur end: mark the first note that comes after the slur region
                            if node.col > *end_col && 
                               (node.col - *end_col) <= 3 {  // Within reasonable distance (3 columns)
                                node.slur_end = Some(true);
                                break;
                            }
                            // Also check if note is within the slur region (fallback)
                            else if node.col >= *start_col && node.col <= *end_col {
                                // Mark this as a potential slur end - we'll find the rightmost one
                                node.slur_end = Some(true);
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Recursively process child nodes
        mark_slur_end_nodes(&mut node.nodes, slur_regions);
    }
    
    // Now clean up - only keep the rightmost slur_end for each slur region
    clean_up_slur_ends(nodes, slur_regions);
}

fn clean_up_slur_ends(nodes: &mut [Node], slur_regions: &HashMap<usize, Vec<(usize, usize)>>) {
    let mut rightmost_by_region: HashMap<(usize, usize, usize), usize> = HashMap::new();
    
    // Find the rightmost pitch for each slur region
    find_rightmost_pitch_in_regions(nodes, slur_regions, &mut rightmost_by_region);
    
    // Clear all slur_end flags except for the rightmost pitches
    clear_non_rightmost_slur_ends(nodes, slur_regions, &rightmost_by_region);
}

fn find_rightmost_pitch_in_regions(nodes: &[Node], slur_regions: &HashMap<usize, Vec<(usize, usize)>>, rightmost_by_region: &mut HashMap<(usize, usize, usize), usize>) {
    for node in nodes {
        if node.node_type == "PITCH" && node.pitch_code.is_some() && node.slur_end == Some(true) {
            // Look for slur regions on lines above (check up to 3 lines above)
            for distance in 1..=3 {
                if let Some(check_line) = node.row.checked_sub(distance) {
                    if let Some(regions) = slur_regions.get(&check_line) {
                        for (start_col, end_col) in regions {
                            // Handle pitches that come after the slur region
                            if node.col > *end_col && (node.col - *end_col) <= 3 {
                                let key = (check_line, *start_col, *end_col);
                                let current_min = rightmost_by_region.get(&key).unwrap_or(&usize::MAX);
                                // For post-slur pitches, we want the LEFTMOST (first) one
                                if node.col < *current_min {
                                    rightmost_by_region.insert(key, node.col);
                                }
                                break;
                            }
                            // Handle pitches within the slur region (fallback case)
                            else if node.col >= *start_col && node.col <= *end_col {
                                let key = (check_line, *start_col, *end_col);
                                let current_max = rightmost_by_region.get(&key).unwrap_or(&0);
                                if node.col >= *current_max {
                                    rightmost_by_region.insert(key, node.col);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        find_rightmost_pitch_in_regions(&node.nodes, slur_regions, rightmost_by_region);
    }
}

fn clear_non_rightmost_slur_ends(nodes: &mut [Node], slur_regions: &HashMap<usize, Vec<(usize, usize)>>, rightmost_by_region: &HashMap<(usize, usize, usize), usize>) {
    for node in nodes {
        if node.node_type == "PITCH" && node.pitch_code.is_some() && node.slur_end == Some(true) {
            // Look for slur regions on lines above (check up to 3 lines above)
            for distance in 1..=3 {
                if let Some(check_line) = node.row.checked_sub(distance) {
                    if let Some(regions) = slur_regions.get(&check_line) {
                        for (start_col, end_col) in regions {
                            // Handle pitches that come after the slur region
                            if node.col > *end_col && (node.col - *end_col) <= 3 {
                                let key = (check_line, *start_col, *end_col);
                                if let Some(target_col) = rightmost_by_region.get(&key) {
                                    if node.col != *target_col {
                                        node.slur_end = None; // Clear non-target slur ends
                                    }
                                }
                                break;
                            }
                            // Handle pitches within the slur region (fallback case)  
                            else if node.col >= *start_col && node.col <= *end_col {
                                let key = (check_line, *start_col, *end_col);
                                if let Some(rightmost_col) = rightmost_by_region.get(&key) {
                                    if node.col != *rightmost_col {
                                        node.slur_end = None; // Clear non-rightmost slur ends
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        clear_non_rightmost_slur_ends(&mut node.nodes, slur_regions, rightmost_by_region);
    }
}