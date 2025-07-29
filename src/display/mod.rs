// src/display/mod.rs  
// Extracted display/rendering functionality from main.rs - no logic changes

use std::collections::{HashMap, HashSet};
use std::fs;
use regex::Regex;
use colored::*;
use crate::models::{Document, Node, LineInfo, Metadata};

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
        
        // Find LINE node for this line number and add its children
        if let Some(line_node) = document.nodes.iter().find(|n| n.node_type == "LINE" && n.row == line_info.line_number) {
            // Only collect nodes that actually belong to this line (not child nodes from other lines)
            collect_line_content_for_line(&line_node.nodes, &mut line_nodes, line_info.line_number);
        }
        
        // Also collect any nodes from other lines that have children positioned on this line
        for line_node in &document.nodes {
            if line_node.node_type == "LINE" && line_node.row != line_info.line_number {
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
                let colored_title = match color.as_str() {
                    "yellow" => display_value.yellow().bold().underline(),
                    "white" => display_value.white().bold().underline(),
                    "green" => display_value.green().bold().underline(),
                    "darkcyan" => display_value.cyan().bold().underline(),
                    "red" => display_value.red().bold().underline(),
                    "magenta" => display_value.magenta().bold().underline(),
                    "blue" => display_value.blue().bold().underline(),
                    "brown" => display_value.truecolor(165, 42, 42).bold().underline(),
                    _ => display_value.normal().bold().underline(),
                };
                line_output.push_str(&colored_title.to_string());
            } else if is_beat_element {
                // Apply underline to beat elements
                let colored_val = match color.as_str() {
                    "yellow" => display_value.yellow().underline(),
                    "white" => display_value.white().underline(),
                    "green" => display_value.green().underline(),
                    "darkcyan" => display_value.cyan().underline(),
                    "red" => display_value.red().underline(),
                    "magenta" => display_value.magenta().underline(),
                    "blue" => display_value.blue().underline(),
                    "brown" => display_value.truecolor(165, 42, 42).underline(),
                    _ => display_value.normal().underline(),
                };
                if reverse {
                    line_output.push_str(&colored_val.on_truecolor(50, 50, 50).to_string());
                } else {
                    line_output.push_str(&colored_val.to_string());
                }
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

pub fn parse_css_for_ansi(css_path: &str) -> HashMap<String, (String, bool)> {
    let mut styles = HashMap::new();
    let content = fs::read_to_string(css_path).unwrap_or_default();
    let rule_regex =
        Regex::new(r"\.token-([a-zA-Z0-9_-]+)\s*\{\s*color:\s*([a-zA-Z]+)\s*;(?:\s*/\*\s*(.*?)\s*\*/)?\s*\}")
            .unwrap();

    for cap in rule_regex.captures_iter(&content) {
        let token_name = cap[1].to_uppercase().replace("-", "_");
        let color = cap[2].to_lowercase();
        let reverse = cap.get(3).map_or(false, |m| m.as_str().contains("reverse"));
        styles.insert(token_name, (color, reverse));
    }
    styles
}

pub fn colorize_string(s: &str, color: &str, reverse: bool) -> String {
    let mut colored_s = match color {
        "yellow" => s.yellow(),
        "white" => s.white(),
        "green" => s.green(),
        "darkcyan" => s.cyan(),
        "red" => s.red(),
        "magenta" => s.magenta(),
        "blue" => s.blue(),
        "brown" => s.truecolor(165, 142, 142),
        _ => s.normal(),
    };
    if reverse {
        colored_s = colored_s.on_truecolor(50, 50, 50); // Dark grey background for reverse
    }
    colored_s.to_string()
}

pub fn generate_legend_string(
    styles: &HashMap<String, (String, bool)>,
    used_tokens: &HashMap<String, String>,
    metadata: Option<&Metadata>,
    for_flattener: bool,
) -> String {
    let mut legend = String::new();
    legend.push_str(&format!("{}\n", "--- Active Token Legend ---".bold()));
    let mut sorted_tokens: Vec<_> = used_tokens.iter().collect();
    sorted_tokens.sort_by_key(|(k, _v)| *k);

    for (token_type, sample_value) in sorted_tokens {
        if let Some((color, reverse)) = styles.get(token_type as &str) {
            legend.push_str(&format!(
                "- {}: {}\n",
                token_type,
                colorize_string(sample_value, color, *reverse)
            ));
        }
    }

    if let Some(meta) = metadata {
        if meta.title.is_some() {
            if let Some((color, _)) = styles.get("TITLE") {
                legend.push_str(&format!(
                    "- {}: {}\n",
                    "TITLE",
                    colorize_string("Title Text", color, false).bold().underline()
                ));
            }
        }
        if !meta.directives.is_empty() {
            if let Some((color, _)) = styles.get("DIRECTIVE_KEY") {
                legend.push_str(&format!(
                    "- {}: {}\n",
                    "DIRECTIVE_KEY",
                    colorize_string("key:", color, false)
                ));
            }
            if let Some((color, _)) = styles.get("DIRECTIVE_VALUE") {
                legend.push_str(&format!(
                    "- {}: {}\n",
                    "DIRECTIVE_VALUE",
                    colorize_string("value", color, false)
                ));
            }
        }
    }

    if for_flattener {
        legend.push_str(&format!(
            "- {}: {}\n",
            "UNASSIGNED",
            colorize_string(" ", "white", true)
        ));
    }

    legend.push_str(&format!("{}\n", "---------------------------".bold()));
    legend
}