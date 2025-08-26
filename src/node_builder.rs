// Node Builder Module  
// Handles Phase 1 of spatial analysis: converting tokens to parsed elements
// with vertical attachments (octave markers, ornaments, etc.)

use std::collections::{HashMap, HashSet};
use crate::models::{LineInfo, Token, Node}; // Keep Node for legacy function
use crate::models_v2::{ParsedElement, ParsedChild, Position, OrnamentType};
use crate::pitch::{lookup_pitch, guess_notation};

fn is_dash(value: &str) -> bool {
    value.chars().all(|c| c == '-')
}

#[derive(Debug)]
struct AttachedToken {
    token: Token,
    distance: i8, // vertical distance from pitch (-1 = above, +1 = below, etc)
}

fn should_treat_as_lyrics(line_tokens: &[&Token]) -> bool {
    // If all tokens are WORD tokens, it's likely lyrics
    line_tokens.iter().all(|t| t.token_type == "WORD") && 
    !line_tokens.is_empty()
}

/// New version that produces ParsedElements
pub fn attach_floating_elements_v2(physical_tokens: &[Token], lines_info: &[LineInfo], raw_text: &str) -> (Vec<ParsedElement>, HashSet<(usize, usize)>) {
    let mut elements = Vec::new();
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
        let upper_bound = (0..*line_num).rev()
            .find(|&l| main_lines.contains(&l))
            .map_or(0, |l| l + 1);
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

    // First pass: create pitch-anchored groups using two-phase approach
    for token in physical_tokens.iter().filter(|t| main_lines.contains(&t.line)) {
        if token.token_type == "PITCH" && !consumed_coords.contains(&(token.line, token.col)) && !is_dash(&token.value) {
            consumed_coords.insert((token.line, token.col));
            
            // Phase 1: Collect all tokens at this column with their vertical distances
            let mut attached_tokens = Vec::new();
            
            // Check lines above (stop at empty lines or line boundaries)
            if token.line > 0 {
                let mut line_above = token.line - 1;
                loop {
                // Check if this line has any content (not empty)
                let has_content = physical_tokens.iter().any(|t| 
                    t.line == line_above && 
                    t.token_type != "WHITESPACE" && 
                    t.token_type != "NEWLINE"
                );
                
                if !has_content {
                    break; // Stop at empty line
                }
                
                // Look for token at same column
                if let Some(t) = token_map.get(&(line_above, token.col)) {
                    if !consumed_coords.contains(&(t.line, t.col)) {
                        let distance = line_above as i8 - token.line as i8;
                        attached_tokens.push(AttachedToken {
                            token: (*t).clone(),
                            distance,
                        });
                        consumed_coords.insert((t.line, t.col));
                    }
                }
                
                if line_above == 0 {
                    break;
                }
                line_above -= 1;
                
                // Stop if we've reached the boundary
                if let Some((upper, _)) = boundaries.get(&token.line) {
                    if line_above < *upper {
                        break;
                    }
                }
                }
            }
            
            // Check lines below
            let mut line_below = token.line + 1;
            while line_below < lines_info.len() {
                // Check if this line has any content (not empty)
                let has_content = physical_tokens.iter().any(|t| 
                    t.line == line_below && 
                    t.token_type != "WHITESPACE" && 
                    t.token_type != "NEWLINE"
                );
                
                if !has_content {
                    break; // Stop at empty line
                }
                
                // Look for token at same column
                if let Some(t) = token_map.get(&(line_below, token.col)) {
                    if !consumed_coords.contains(&(t.line, t.col)) {
                        let distance = line_below as i8 - token.line as i8;
                        attached_tokens.push(AttachedToken {
                            token: (*t).clone(),
                            distance,
                        });
                        consumed_coords.insert((t.line, t.col));
                    }
                }
                
                line_below += 1;
                
                // Stop if we've reached the boundary
                if let Some((_, lower)) = boundaries.get(&token.line) {
                    if line_below > *lower {
                        break;
                    }
                }
            }
            
            // Phase 2: Process the attached tokens and create child elements
            let mut children = Vec::new();
            
            for attached in attached_tokens {
                match (attached.token.token_type.as_str(), attached.token.value.as_str(), attached.distance) {
                    // Octave markers
                    ("SYMBOLS", ".", distance) | ("SYMBOLS", "'", distance) | ("SYMBOLS", ":", distance) => {
                        children.push(ParsedChild::OctaveMarker {
                            symbol: attached.token.value,
                            distance,
                        });
                    },
                    // Mordent indicators
                    ("SYMBOLS", "m", -1) => {
                        children.push(ParsedChild::Ornament {
                            kind: OrnamentType::Mordent,
                            distance: attached.distance,
                        });
                    },
                    // Lyrics
                    ("WORD", _, 1..) => {
                        children.push(ParsedChild::Syllable {
                            text: attached.token.value,
                            distance: attached.distance,
                        });
                    },
                    // Skip underscores - they are handled separately as slur regions
                    ("SYMBOLS", "_", _) => {
                        // Do nothing - underscores are processed as slur regions by the slur logic
                    },
                    // Other tokens - skip for now (could add more child types later)
                    _ => {}
                }
            }
            
            // Calculate pitch and octave - with fallback trying all notation systems
            let pitch_code = lookup_pitch(&token.value, notation).or_else(|| {
                // Try other notation systems if the detected one fails
                use crate::pitch::Notation;
                lookup_pitch(&token.value, Notation::Sargam)
                    .or_else(|| lookup_pitch(&token.value, Notation::Western))
                    .or_else(|| lookup_pitch(&token.value, Notation::Number))
            }).unwrap_or_else(|| {
                eprintln!("V2 NODE BUILDER: lookup_pitch failed for '{}' with all notations, defaulting to N1", token.value);
                crate::pitch::PitchCode::N1 // Default to C/Sa/1
            });
            
            // Calculate octave from attached octave marker children
            let mut octave = 0i8; // default to middle octave
            for child in &children {
                if let ParsedChild::OctaveMarker { symbol, distance } = child {
                    octave += match symbol.as_str() {
                        "." | ":" if *distance < 0 => 1,  // above means upper octave
                        "." | ":" if *distance > 0 => -1,  // below means lower octave
                        _ => 0,
                    };
                }
            }
            
            elements.push(ParsedElement::Note {
                pitch_code,
                octave,
                value: token.value.clone(),
                position: Position::new(token.line, token.col),
                children,
            });
        }
    }
    
    // Process any remaining lines that look like lyrics but weren't attached to pitches
    let raw_lines: Vec<&str> = raw_text.lines().collect();
    let mut processed_lyrics_lines = HashSet::new();
    
    for line_num in 0..raw_lines.len() {
        // Skip if already processed
        if processed_lyrics_lines.contains(&line_num) {
            continue;
        }
        
        // Get all tokens on this line that haven't been consumed
        let line_tokens: Vec<&Token> = physical_tokens.iter()
            .filter(|t| t.line == line_num && !consumed_coords.contains(&(t.line, t.col)))
            .collect();
        
        if !line_tokens.is_empty() && should_treat_as_lyrics(&line_tokens) {
            // Re-parse this line as lyrics
            if let Some(raw_line) = raw_lines.get(line_num) {
                let word_tokens = crate::lyrics::parse_text_as_word_tokens(raw_line, line_num);
                
                // Mark all original tokens as consumed
                for token in line_tokens {
                    consumed_coords.insert((token.line, token.col));
                }
                
                // Add the word tokens as standalone elements
                for word_token in word_tokens {
                    elements.push(ParsedElement::Word {
                        text: word_token.value,
                        position: Position::new(word_token.line, word_token.col),
                    });
                }
                
                processed_lyrics_lines.insert(line_num);
            }
        }
    }
    
    // Second pass: add all other tokens as elements
    for token in physical_tokens {
        if !consumed_coords.contains(&(token.line, token.col)) {
            consumed_coords.insert((token.line, token.col));
            
            let position = Position::new(token.line, token.col);
            
            match token.token_type.as_str() {
                "DASH" => {
                    // For DASH tokens, calculate pitch and octave by looking at adjacent tokens
                    let pitch_token = physical_tokens.iter()
                        .filter(|t| t.line == token.line && t.col < token.col && t.token_type == "PITCH" && !is_dash(&t.value))
                        .max_by_key(|t| t.col);
                    
                    let (pitch_code, octave) = if let Some(pitch_token) = pitch_token {
                        if let Some(pitch_code) = lookup_pitch(&pitch_token.value, notation) {
                            // Calculate octave by checking for octave markers above and below this DASH token
                            let mut octave = 0i8;
                            
                            // Check row above (upper octave markers)
                            if let Some(above_token) = physical_tokens.iter().find(|t|
                                t.line == token.line.saturating_sub(1) && 
                                t.col == token.col && 
                                t.token_type == "SYMBOLS" &&
                                (t.value == "." || t.value == "'" || t.value == ":")
                            ) {
                                octave += match above_token.value.as_str() {
                                    "." | "'" | ":" => 1, // above means upper octave
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
                                    "." | "'" | ":" => -1, // below means lower octave
                                    _ => 0,
                                };
                            }
                            
                            (Some(pitch_code), Some(octave))
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    };
                    
                    elements.push(ParsedElement::Dash { pitch_code, octave, position });
                },
                "REST" => {
                    elements.push(ParsedElement::Rest {
                        value: token.value.clone(),
                        position,
                    });
                },
                "BARLINE" => {
                    elements.push(ParsedElement::Barline {
                        style: token.value.clone(),
                        position,
                    });
                },
                "WHITESPACE" => {
                    elements.push(ParsedElement::Whitespace {
                        width: token.value.len(),
                        position,
                    });
                },
                "NEWLINE" => {
                    elements.push(ParsedElement::Newline { position });
                },
                "WORD" => {
                    elements.push(ParsedElement::Word {
                        text: token.value.clone(),
                        position,
                    });
                },
                "SYMBOLS" => {
                    elements.push(ParsedElement::Symbol {
                        value: token.value.clone(),
                        position,
                    });
                },
                _ => {
                    elements.push(ParsedElement::Unknown {
                        value: token.value.clone(),
                        position,
                    });
                }
            }
        }
    }

    // Sort elements by position (row, then column)
    elements.sort_by(|a, b| {
        let pos_a = a.position();
        let pos_b = b.position();
        pos_a.row.cmp(&pos_b.row).then_with(|| pos_a.col.cmp(&pos_b.col))
    });

    (elements, consumed_coords)
}

/// Legacy version that produces Nodes (will be removed after migration)
pub fn attach_floating_elements(physical_tokens: &[Token], lines_info: &[LineInfo], raw_text: &str) -> (Vec<Node>, HashSet<(usize, usize)>) {
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
        let upper_bound = (0..*line_num).rev()
            .find(|&l| main_lines.contains(&l))
            .map_or(0, |l| l + 1);
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

    // First pass: create pitch-anchored groups using two-phase approach
    for token in physical_tokens.iter().filter(|t| main_lines.contains(&t.line)) {
        if token.token_type == "PITCH" && !consumed_coords.contains(&(token.line, token.col)) && !is_dash(&token.value) {
            consumed_coords.insert((token.line, token.col));
            
            // Phase 1: Collect all tokens at this column with their vertical distances
            let mut attached_tokens = Vec::new();
            
            // Check lines above (stop at empty lines or line boundaries)
            if token.line > 0 {
                let mut line_above = token.line - 1;
                loop {
                // Check if this line has any content (not empty)
                let has_content = physical_tokens.iter().any(|t| 
                    t.line == line_above && 
                    t.token_type != "WHITESPACE" && 
                    t.token_type != "NEWLINE"
                );
                
                if !has_content {
                    break; // Stop at empty line
                }
                
                // Look for token at same column
                if let Some(t) = token_map.get(&(line_above, token.col)) {
                    if !consumed_coords.contains(&(t.line, t.col)) {
                        let distance = line_above as i8 - token.line as i8;
                        attached_tokens.push(AttachedToken {
                            token: (*t).clone(),
                            distance,
                        });
                        consumed_coords.insert((t.line, t.col));
                    }
                }
                
                if line_above == 0 {
                    break;
                }
                line_above -= 1;
                
                // Stop if we've reached the boundary
                if let Some((upper, _)) = boundaries.get(&token.line) {
                    if line_above < *upper {
                        break;
                    }
                }
                }
            }
            
            // Check lines below
            let mut line_below = token.line + 1;
            while line_below < lines_info.len() {
                // Check if this line has any content (not empty)
                let has_content = physical_tokens.iter().any(|t| 
                    t.line == line_below && 
                    t.token_type != "WHITESPACE" && 
                    t.token_type != "NEWLINE"
                );
                
                if !has_content {
                    break; // Stop at empty line
                }
                
                // Look for token at same column
                if let Some(t) = token_map.get(&(line_below, token.col)) {
                    if !consumed_coords.contains(&(t.line, t.col)) {
                        let distance = line_below as i8 - token.line as i8;
                        attached_tokens.push(AttachedToken {
                            token: (*t).clone(),
                            distance,
                        });
                        consumed_coords.insert((t.line, t.col));
                    }
                }
                
                line_below += 1;
                
                // Stop if we've reached the boundary
                if let Some((_, lower)) = boundaries.get(&token.line) {
                    if line_below > *lower {
                        break;
                    }
                }
            }
            
            // Phase 2: Process the attached tokens and create child nodes
            let mut child_nodes = Vec::new();
            
            for attached in attached_tokens {
                match (attached.token.token_type.as_str(), attached.token.value.as_str(), attached.distance) {
                    // Octave markers
                    ("SYMBOLS", ".", distance) | ("SYMBOLS", "'", distance) | ("SYMBOLS", ":", distance) => {
                        child_nodes.push(Node::new(
                            "OCTAVE_MARKER".to_string(),
                            attached.token.value,
                            attached.token.line,
                            attached.token.col,
                        ));
                    },
                    // Mordent indicators
                    ("SYMBOLS", "m", -1) => {
                        child_nodes.push(Node::new(
                            "MORDENT".to_string(),
                            attached.token.value,
                            attached.token.line,
                            attached.token.col,
                        ));
                    },
                    // Lyrics
                    ("WORD", _, 1..) => {
                        child_nodes.push(Node::new(
                            "SYL".to_string(),
                            attached.token.value,
                            attached.token.line,
                            attached.token.col,
                        ));
                    },
                    // Skip underscores - they are handled separately as slur regions, not individual attachments
                    ("SYMBOLS", "_", _) => {
                        // Do nothing - underscores are processed as slur regions by the slur logic
                    },
                    // Other tokens - preserve as-is for now
                    _ => {
                        child_nodes.push(Node::new(
                            attached.token.token_type,
                            attached.token.value,
                            attached.token.line,
                            attached.token.col,
                        ));
                    }
                }
            }
            
            let (pitch_code, octave) = if token.token_type == "PITCH" && !is_dash(&token.value) {
                let pitch_code = lookup_pitch(&token.value, notation);
                
                // Calculate octave from attached OCTAVE_MARKER children
                let mut octave = 0i8; // default to middle octave
                for child in &child_nodes {
                    if child.node_type == "OCTAVE_MARKER" {
                        let child_distance = child.row as i8 - token.line as i8;
                        octave += match child.value.as_str() {
                            "." | ":" if child_distance < 0 => 1,  // above means upper octave
                            "." | ":" if child_distance > 0 => -1,  // below means lower octave
                            _ => 0,
                        };
                    }
                }
                
                (pitch_code, Some(octave))
            } else if token.token_type == "DASH" {
                // DASH tokens don't have pitch or octave
                (None, None)
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
    
    // Process any remaining lines that look like lyrics but weren't attached to pitches
    // This handles cases where lyrics appear without musical notation above them
    let raw_lines: Vec<&str> = raw_text.lines().collect();
    let mut processed_lyrics_lines = HashSet::new();
    
    for line_num in 0..raw_lines.len() {
        // Skip if already processed
        if processed_lyrics_lines.contains(&line_num) {
            continue;
        }
        
        // Get all tokens on this line that haven't been consumed
        let line_tokens: Vec<&Token> = physical_tokens.iter()
            .filter(|t| t.line == line_num && !consumed_coords.contains(&(t.line, t.col)))
            .collect();
        
        if !line_tokens.is_empty() && should_treat_as_lyrics(&line_tokens) {
            // Re-parse this line as lyrics
            if let Some(raw_line) = raw_lines.get(line_num) {
                let word_tokens = crate::lyrics::parse_text_as_word_tokens(raw_line, line_num);
                
                // Mark all original tokens as consumed
                for token in line_tokens {
                    consumed_coords.insert((token.line, token.col));
                }
                
                // Add the word tokens as standalone nodes
                for word_token in word_tokens {
                    let node = Node::new(
                        "WORD".to_string(),
                        word_token.value,
                        word_token.line,
                        word_token.col,
                    );
                    nodes.push(node);
                }
                
                processed_lyrics_lines.insert(line_num);
            }
        }
    }
    
    // Second pass: add all other tokens as top-level nodes (including WHITESPACE and BARLINE)
    for token in physical_tokens {
        if !consumed_coords.contains(&(token.line, token.col)) {
            consumed_coords.insert((token.line, token.col));
            
            // For DASH tokens, calculate pitch and octave by looking at adjacent tokens
            let (pitch_code, octave) = if token.token_type == "DASH" {
                // Look for a pitch token in the same line to the left
                let pitch_token = physical_tokens.iter()
                    .filter(|t| t.line == token.line && t.col < token.col && t.token_type == "PITCH" && !is_dash(&t.value))
                    .max_by_key(|t| t.col);
                
                if let Some(pitch_token) = pitch_token {
                    let pitch_code = lookup_pitch(&pitch_token.value, notation);
                    
                    // Calculate octave by checking for octave markers above and below this DASH token
                    let mut octave = 0i8;
                    
                    // Check row above (upper octave markers)
                    if let Some(above_token) = physical_tokens.iter().find(|t|
                        t.line == token.line.saturating_sub(1) && 
                        t.col == token.col && 
                        t.token_type == "SYMBOLS" &&
                        (t.value == "." || t.value == "'" || t.value == ":")
                    ) {
                        octave += match above_token.value.as_str() {
                            "." => 1, // dot above means upper octave
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
                }
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

    (nodes, consumed_coords)
}