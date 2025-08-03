// src/formatter.rs

use crate::models::{Document, Node};
use crate::pitch::{PitchCode, Notation};
use std::collections::HashMap;

fn convert_pitch_code_to_notation(pitch_code: PitchCode, notation: Notation) -> String {
    match notation {
        Notation::Western => match pitch_code {
            PitchCode::N1bb => "Cbb".to_string(),
            PitchCode::N1b => "Cb".to_string(),
            PitchCode::N1 => "C".to_string(),
            PitchCode::N1s => "C#".to_string(),
            PitchCode::N1ss => "C##".to_string(),
            PitchCode::N2bb => "Dbb".to_string(),
            PitchCode::N2b => "Db".to_string(),
            PitchCode::N2 => "D".to_string(),
            PitchCode::N2s => "D#".to_string(),
            PitchCode::N2ss => "D##".to_string(),
            PitchCode::N3bb => "Ebb".to_string(),
            PitchCode::N3b => "Eb".to_string(),
            PitchCode::N3 => "E".to_string(),
            PitchCode::N3s => "E#".to_string(),
            PitchCode::N3ss => "E##".to_string(),
            PitchCode::N4bb => "Fbb".to_string(),
            PitchCode::N4b => "Fb".to_string(),
            PitchCode::N4 => "F".to_string(),
            PitchCode::N4s => "F#".to_string(),
            PitchCode::N4ss => "F##".to_string(),
            PitchCode::N5bb => "Gbb".to_string(),
            PitchCode::N5b => "Gb".to_string(),
            PitchCode::N5 => "G".to_string(),
            PitchCode::N5s => "G#".to_string(),
            PitchCode::N5ss => "G##".to_string(),
            PitchCode::N6bb => "Abb".to_string(),
            PitchCode::N6b => "Ab".to_string(),
            PitchCode::N6 => "A".to_string(),
            PitchCode::N6s => "A#".to_string(),
            PitchCode::N6ss => "A##".to_string(),
            PitchCode::N7bb => "Bbb".to_string(),
            PitchCode::N7b => "Bb".to_string(),
            PitchCode::N7 => "B".to_string(),
            PitchCode::N7s => "B#".to_string(),
            PitchCode::N7ss => "B##".to_string(),
        },
        Notation::Number => match pitch_code {
            PitchCode::N1bb => "1bb".to_string(),
            PitchCode::N1b => "1b".to_string(),
            PitchCode::N1 => "1".to_string(),
            PitchCode::N1s => "1#".to_string(),
            PitchCode::N1ss => "1##".to_string(),
            PitchCode::N2bb => "2bb".to_string(),
            PitchCode::N2b => "2b".to_string(),
            PitchCode::N2 => "2".to_string(),
            PitchCode::N2s => "2#".to_string(),
            PitchCode::N2ss => "2##".to_string(),
            PitchCode::N3bb => "3bb".to_string(),
            PitchCode::N3b => "3b".to_string(),
            PitchCode::N3 => "3".to_string(),
            PitchCode::N3s => "3#".to_string(),
            PitchCode::N3ss => "3##".to_string(),
            PitchCode::N4bb => "4bb".to_string(),
            PitchCode::N4b => "4b".to_string(),
            PitchCode::N4 => "4".to_string(),
            PitchCode::N4s => "4#".to_string(),
            PitchCode::N4ss => "4##".to_string(),
            PitchCode::N5bb => "5bb".to_string(),
            PitchCode::N5b => "5b".to_string(),
            PitchCode::N5 => "5".to_string(),
            PitchCode::N5s => "5#".to_string(),
            PitchCode::N5ss => "5##".to_string(),
            PitchCode::N6bb => "6bb".to_string(),
            PitchCode::N6b => "6b".to_string(),
            PitchCode::N6 => "6".to_string(),
            PitchCode::N6s => "6#".to_string(),
            PitchCode::N6ss => "6##".to_string(),
            PitchCode::N7bb => "7bb".to_string(),
            PitchCode::N7b => "7b".to_string(),
            PitchCode::N7 => "7".to_string(),
            PitchCode::N7s => "7#".to_string(),
            PitchCode::N7ss => "7##".to_string(),
        },
        Notation::Sargam => match pitch_code {
            PitchCode::N1bb => "Sbb".to_string(),
            PitchCode::N1b => "Sb".to_string(),
            PitchCode::N1 => "S".to_string(),
            PitchCode::N1s => "S#".to_string(),
            PitchCode::N1ss => "S##".to_string(),
            PitchCode::N2bb => "Rbb".to_string(),
            PitchCode::N2b => "r".to_string(), // komal Re
            PitchCode::N2 => "R".to_string(),
            PitchCode::N2s => "R#".to_string(),
            PitchCode::N2ss => "R##".to_string(),
            PitchCode::N3bb => "Gbb".to_string(),
            PitchCode::N3b => "g".to_string(), // komal Ga
            PitchCode::N3 => "G".to_string(),
            PitchCode::N3s => "G#".to_string(),
            PitchCode::N3ss => "G##".to_string(),
            PitchCode::N4bb => "mbb".to_string(),
            PitchCode::N4b => "mb".to_string(),
            PitchCode::N4 => "m".to_string(), // shuddha Ma
            PitchCode::N4s => "M".to_string(), // tivra Ma
            PitchCode::N4ss => "M#".to_string(),
            PitchCode::N5bb => "Pbb".to_string(),
            PitchCode::N5b => "Pb".to_string(),
            PitchCode::N5 => "P".to_string(),
            PitchCode::N5s => "P#".to_string(),
            PitchCode::N5ss => "P##".to_string(),
            PitchCode::N6bb => "Dbb".to_string(),
            PitchCode::N6b => "d".to_string(), // komal Dha
            PitchCode::N6 => "D".to_string(),
            PitchCode::N6s => "D#".to_string(),
            PitchCode::N6ss => "D##".to_string(),
            PitchCode::N7bb => "Nbb".to_string(),
            PitchCode::N7b => "n".to_string(), // komal Ni
            PitchCode::N7 => "N".to_string(),
            PitchCode::N7s => "N#".to_string(),
            PitchCode::N7ss => "N##".to_string(),
        },
    }
}

pub fn format_document_to_text(document: &Document) -> String {
    let mut lines: HashMap<usize, String> = HashMap::new();
    let mut max_line = 0;

    // First, populate with metadata
    if let Some(title) = &document.metadata.title {
        let line = lines.entry(title.row).or_insert_with(String::new);
        pad_line(line, title.col);
        line.push_str(&title.text);
        max_line = max_line.max(title.row);
    }
    for directive in &document.metadata.directives {
        let line = lines.entry(directive.row).or_insert_with(String::new);
        pad_line(line, directive.col);
        line.push_str(&format!("{}: {}", directive.key, directive.value));
        max_line = max_line.max(directive.row);
    }

    // Then, populate with nodes
    for node in &document.nodes {
        populate_lines_from_node(node, &mut lines, &mut max_line, document);
    }

    if max_line == 0 && lines.is_empty() {
        return String::new();
    }

    let mut result = Vec::new();
    // Find the minimum line number with content to avoid adding empty lines at the start
    let min_line = if lines.is_empty() { 
        0 
    } else { 
        *lines.keys().min().unwrap_or(&0) 
    };
    
    // Loop from min_line to max_line inclusive
    for i in min_line..=max_line {
        result.push(lines.get(&i).cloned().unwrap_or_default());
    }

    let mut formatted_text = result.join("\n");
    
    // Count total NEWLINE tokens in the document
    let newline_count = count_newline_tokens(&document.nodes);
    
    // Count newlines already in formatted text
    let existing_newlines = formatted_text.matches('\n').count();
    
    
    // Add missing newlines
    let missing_newlines = newline_count.saturating_sub(existing_newlines);
    for _ in 0..missing_newlines {
        formatted_text.push('\n');
    }
    
    formatted_text
}

fn populate_lines_from_node(node: &Node, lines: &mut HashMap<usize, String>, max_line: &mut usize, document: &Document) {
    // Acknowledge the row number of all nodes to correctly calculate the maximum line number
    *max_line = (*max_line).max(node.row);

    // Handle NEWLINE tokens specially - they ensure empty lines exist in output
    if node.node_type == "NEWLINE" {
        // Ensure the line exists in the HashMap, even if it's empty
        lines.entry(node.row).or_insert_with(String::new);
        return; // Don't process children for NEWLINE tokens
    }
    
    // Skip container/structural nodes that don't have direct textual representation
    if node.node_type != "LINE" && node.node_type != "MUSICAL_LINE" && node.node_type != "BEAT" {
        let line = lines.entry(node.row).or_insert_with(String::new);
        pad_line(line, node.col);
        
        // Determine the value to write - convert pitch codes if notation_system is set
        let value_to_write = if node.node_type == "PITCH" && node.pitch_code.is_some() {
            if let Some(ref notation_system) = document.notation_system {
                let target_notation = match notation_system.as_str() {
                    "Western" => Notation::Western,
                    "Number" => Notation::Number,
                    "Sargam" => Notation::Sargam,
                    _ => {
                        // Use detected system from metadata as fallback
                        if let Some(ref detected) = document.metadata.detected_system {
                            match detected.as_str() {
                                "Western" => Notation::Western,
                                "Number" => Notation::Number,
                                _ => Notation::Sargam,
                            }
                        } else {
                            Notation::Sargam
                        }
                    }
                };
                convert_pitch_code_to_notation(node.pitch_code.unwrap(), target_notation)
            } else {
                // Use original value if no notation_system override
                node.value.clone()
            }
        } else {
            // Non-pitch nodes use original value
            node.value.clone()
        };
        
        // Overwrite the specific part of the line with the value
        let value_len = value_to_write.chars().count();
        if line.chars().count() > node.col {
            let mut new_line = line.chars().collect::<Vec<char>>();
            let end = (node.col + value_len).min(new_line.len());
            new_line.splice(node.col..end, value_to_write.chars());
            *line = new_line.into_iter().collect();
        } else {
            line.push_str(&value_to_write);
        }
    }

    for child in &node.nodes {
        populate_lines_from_node(child, lines, max_line, document);
    }
}

fn pad_line(line: &mut String, target_col: usize) {
    while line.len() < target_col {
        line.push(' ');
    }
}

fn _has_newline_in_tree(node: &Node) -> bool {
    node.node_type == "NEWLINE" || 
    node.nodes.iter().any(|child| _has_newline_in_tree(child))
}

fn count_newline_tokens(nodes: &[Node]) -> usize {
    nodes.iter().map(|node| {
        let self_count = if node.node_type == "NEWLINE" { 1 } else { 0 };
        let child_count = count_newline_tokens(&node.nodes);
        self_count + child_count
    }).sum()
}