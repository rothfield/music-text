// src/display/mod.rs  
// Extracted display/rendering functionality from main.rs - no logic changes

use std::collections::{HashMap, HashSet};
use crate::models::{Document, Node, LineInfo};
use crate::colorizer::{colorize_string, colorize_title, colorize_beat_element};

pub fn generate_flattened_spatial_view(
    document: &Document,
    lines_info: &[LineInfo],
    styles: &HashMap<String, (String, bool)>,
    _main_lines: &HashSet<usize>,
) -> String {
    let mut output_lines = Vec::new();
    
    // Create metadata nodes for lines that need them
    let mut metadata_by_line: HashMap<usize, Vec<Node>> = HashMap::new();
    if let Some(title) = &document.metadata.title {
        metadata_by_line.entry(title.row).or_default().push(Node::new(
            "TITLE".to_string(),
            title.text.clone(),
            title.row,
            title.col,
        ));
    }
    for directive in &document.metadata.directives {
        // Skip derived directives that are auto-generated from two-segment extraction
        if directive.key == "Author" || directive.key == "Title" {
            continue;
        }
        
        let key_node = Node::new(
            "DIRECTIVE_KEY".to_string(),
            format!("{}:", directive.key),
            directive.row,
            directive.col,
        );
        let value_node = Node::new(
            "DIRECTIVE_VALUE".to_string(),
            directive.value.clone(),
            directive.row,
            directive.col + directive.key.len() + 1,
        );
        metadata_by_line.entry(directive.row).or_default().push(key_node);
        metadata_by_line.entry(directive.row).or_default().push(value_node);
    }

    // Process each line in order
    for line_info in lines_info {
        let mut line_output = String::new();
        let mut current_col = 0;

        // Get nodes for this line (either from LINE nodes or metadata)
        let mut line_nodes = Vec::new();
        
        // Add metadata nodes if they exist for this line
        if let Some(meta_nodes) = metadata_by_line.get(&line_info.line_number) {
            line_nodes.extend(meta_nodes.iter().cloned());
        }
        
        // Find LINE or MUSICAL_LINE node for this line number and add its children
        if let Some(line_node) = document.nodes.iter().find(|n| (n.node_type == "LINE" || n.node_type == "MUSICAL_LINE") && n.row == line_info.line_number) {
            // Only collect nodes that actually belong to this line (not child nodes from other lines)
            collect_line_content_for_line(&line_node.nodes, &mut line_nodes, line_info.line_number);
        }
        
        // Also collect any nodes from other lines that have children positioned on this line
        for line_node in &document.nodes {
            if (line_node.node_type == "LINE" || line_node.node_type == "MUSICAL_LINE") && line_node.row != line_info.line_number {
                collect_child_nodes_for_line(&line_node.nodes, &mut line_nodes, line_info.line_number);
            }
        }
        
        line_nodes.sort_by_key(|n| n.col);

        for node in line_nodes {
            if node.node_type == "NEWLINE" || node.node_type == "LINE" || node.node_type == "BEAT" {
                continue; // Skip structural nodes
            }

            if node.col > current_col {
                line_output.push_str(&" ".repeat(node.col - current_col));
            }

            let (color, mut reverse) = styles.get(&node.node_type).cloned().unwrap_or_default();
            
            // Check if this is a beat element
            let (is_beat_element, display_value) = if node.value.starts_with("BEAT_ELEMENT:") {
                (true, node.value.strip_prefix("BEAT_ELEMENT:").unwrap_or(&node.value))
            } else {
                (false, node.value.as_str())
            };
            
            if node.node_type == "TITLE" {
                line_output.push_str(&colorize_title(display_value, &color));
            } else if is_beat_element {
                line_output.push_str(&colorize_beat_element(display_value, &color, reverse));
            } else {
                // Only apply reverse styling to the specific "unassigned" token from the input
                if node.value == "unassigned" {
                    reverse = true;
                }
                
                let colored_val = colorize_string(display_value, &color, reverse);
                line_output.push_str(&colored_val);
            }
            current_col = node.col + display_value.len();
        }
        
        if current_col < line_info.line_text.len() {
            line_output.push_str(&" ".repeat(line_info.line_text.len() - current_col));
        }

        output_lines.push(line_output);
    }

    output_lines.join("\n")
}

fn collect_line_content_for_line(nodes: &[Node], result: &mut Vec<Node>, target_line: usize) {
    for node in nodes {
        if node.node_type == "BEAT" {
            // For BEAT nodes, collect their children and mark them as beat elements, but only if they're on the target line
            collect_beat_content_for_line(&node.nodes, result, true, target_line);
        } else if node.node_type != "NEWLINE" && node.row == target_line {
            // Add non-newline nodes directly, but only if they're on the target line
            result.push(node.clone());
            // Also collect any children (like octave markers) that are on the target line
            collect_line_content_for_line(&node.nodes, result, target_line);
        }
    }
}

fn collect_beat_content_for_line(nodes: &[Node], result: &mut Vec<Node>, is_beat_element: bool, target_line: usize) {
    for node in nodes {
        if node.node_type != "NEWLINE" && node.row == target_line {
            let mut beat_node = node.clone();
            // Mark ALL nodes inside a beat for underlining
            if is_beat_element {
                beat_node.value = format!("BEAT_ELEMENT:{}", beat_node.value);
            }
            result.push(beat_node);
            // Also collect any children and mark them as beat elements too
            collect_beat_content_for_line(&node.nodes, result, is_beat_element, target_line);
        }
    }
}

fn collect_child_nodes_for_line(nodes: &[Node], result: &mut Vec<Node>, target_line: usize) {
    for node in nodes {
        if node.node_type == "BEAT" {
            // Look for child nodes in beats that belong to the target line
            collect_child_nodes_for_line(&node.nodes, result, target_line);
        } else {
            // Check if this node has children on the target line
            collect_child_nodes_for_line(&node.nodes, result, target_line);
            // Check if any of the direct children are on the target line
            for child in &node.nodes {
                if child.row == target_line && child.node_type != "NEWLINE" {
                    result.push(child.clone());
                }
            }
        }
    }
}



