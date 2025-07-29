use clap::Parser;
use colored::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use colored::control;
use serde_yaml;

mod structs;
mod lilypond_converter;

use structs::{ChunkInfo, LineInfo, Token, Title, Directive, Metadata, Document, Node, TokenType};
use lilypond_converter::convert_to_lilypond;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file
    input_file: PathBuf,
}

// --- Stage 1.5: Metadata Parser ---
fn parse_metadata(tokens: &[Token]) -> (Metadata, Vec<Token>) {
    let mut metadata = Metadata {
        title: None,
        directives: Vec::new(),
    };
    let mut consumed_lines = HashSet::new();

    // Find the first line containing a barline
    let first_main_line = tokens
        .iter()
        .find(|t| t.token_type == "BARLINE")
        .map(|t| t.line)
        .unwrap_or(usize::MAX);

    // 1. Extract Title: First line of WORDs before any main content
    if let Some(first_word) = tokens
        .iter()
        .find(|t| t.token_type == "WORD" && t.line < first_main_line)
    {
        let title_line = first_word.line;
        metadata.title = Some(Title {
            text: tokens
                .iter()
                .filter(|t| t.line == title_line && t.token_type == "WORD")
                .map(|t| t.value.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            row: title_line,
            col: first_word.col,
        });
        consumed_lines.insert(title_line);
    }

    // 2. Extract Directives: Look for "key: value" or "key-value" patterns
    for (i, token) in tokens.iter().enumerate() {
        if token.line >= first_main_line { continue; }

        if token.token_type == "WORD" {
            if token.value.ends_with(':') {
                let key = token.value.trim_end_matches(':').to_string();
                let value_tokens: Vec<_> = tokens
                    .iter()
                    .skip(i + 1)
                    .take_while(|t| t.line == token.line && t.token_type != "NEWLINE")
                    .filter(|t| t.token_type != "WHITESPACE")
                    .collect();

                if !value_tokens.is_empty() {
                    metadata.directives.push(Directive {
                        key,
                        value: value_tokens.iter().map(|t| t.value.as_str()).collect::<Vec<_>>().join(" "),
                        row: token.line,
                        col: token.col,
                    });
                    consumed_lines.insert(token.line);
                }
            } else if token.value.contains('-') {
                let parts: Vec<&str> = token.value.splitn(2, '-').collect();
                if parts.len() == 2 {
                    metadata.directives.push(Directive {
                        key: parts[0].to_string(),
                        value: parts[1].to_string(),
                        row: token.line,
                        col: token.col,
                    });
                    consumed_lines.insert(token.line);
                }
            }
        }
    }

    // 3. Filter out consumed tokens
    let remaining_tokens: Vec<Token> = tokens
        .iter()
        .filter(|t| !consumed_lines.contains(&t.line))
        .cloned()
        .collect();

    (metadata, remaining_tokens)
}

// --- Stage 2: Flatten Spatial Relationships ---

fn flatten_spatial_relationships(physical_tokens: &[Token], lines_info: &[LineInfo]) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut consumed_coords = HashSet::new();
    let token_map: HashMap<(usize, usize), &Token> =
        physical_tokens.iter().map(|t| ((t.line, t.col), t)).collect();

    let main_lines: HashSet<usize> =
        physical_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();

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
                            child_nodes.push(Node {
                                node_type: "OCTAVE_MARKER".to_string(),
                                value: t.value.clone(),
                                row: t.line,
                                col: t.col,
                                divisions: 0,
                                dash_consumed: false,
                                nodes: Vec::new(),
                            });
                            consumed_coords.insert((t.line, t.col));
                        } else if token_type == "WORD" || token_type == "METADATA" || token_type == "SYMBOLS" {
                             child_nodes.push(Node {
                                node_type: t.token_type.clone(),
                                value: t.value.clone(),
                                row: t.line,
                                col: t.col,
                                divisions: 0,
                                dash_consumed: false,
                                nodes: Vec::new(),
                            });
                            consumed_coords.insert((t.line, t.col));
                        }
                    }
                }
            }
            
            nodes.push(Node {
                node_type: token.token_type.clone(),
                value: token.value.clone(),
                row: token.line,
                col: token.col,
                divisions: 0,
                dash_consumed: false,
                nodes: child_nodes,
            });
        }
    }
    
    // Second pass: add all other tokens as top-level nodes
    for token in physical_tokens {
        if !consumed_coords.contains(&(token.line, token.col)) {
            nodes.push(Node {
                node_type: token.token_type.clone(),
                value: token.value.clone(),
                row: token.line,
                col: token.col,
                divisions: 0,
                dash_consumed: false,
                nodes: Vec::new(),
            });
        }
    }

    nodes.sort_by(|a, b| a.row.cmp(&b.row).then_with(|| a.col.cmp(&b.col)));

    nodes
}

// --- Stage 3: Beat Grouping ---
// NOTE: Must maintain round-trip capability - any transformations must be reversible
// to reconstruct the original input text exactly
fn group_nodes_into_lines_and_beats(nodes: &[Node], lines_of_music: &Vec<usize>) -> Vec<Node> {
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
    
    Node {
        node_type: "LINE".to_string(),
        value: format!("music-line-{}", line_num),
        row: line_num,
        col: start_col,
        divisions: 0, // Lines don't have divisions
        dash_consumed: false,
        nodes: beats_and_separators,
    }
}

fn create_regular_line_node(line_num: usize, line_nodes: Vec<&Node>) -> Node {
    let mut sorted_nodes = line_nodes;
    sorted_nodes.sort_by_key(|n| n.col);
    let start_col = sorted_nodes.first().map(|n| n.col).unwrap_or(0);
    
    Node {
        node_type: "LINE".to_string(),
        value: format!("line-{}", line_num),
        row: line_num,
        col: start_col,
        divisions: 0, // Lines don't have divisions
        dash_consumed: false,
        nodes: sorted_nodes.into_iter().cloned().collect(),
    }
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
    
    Node {
        node_type: "BEAT".to_string(),
        value: format!("beat-{}", total_subdivisions),
        row,
        col: start_col,
        divisions: total_subdivisions,
        dash_consumed: false,
        nodes: elements,
    }
}

fn generate_flattened_spatial_view(
    document: &Document,
    lines_info: &[LineInfo],
    styles: &HashMap<String, (String, bool)>,
    _main_lines: &HashSet<usize>,
) -> String {
    let mut output_lines = Vec::new();
    
    // Create metadata nodes for lines that need them
    let mut metadata_by_line: HashMap<usize, Vec<Node>> = HashMap::new();
    if let Some(title) = &document.metadata.title {
        metadata_by_line.entry(title.row).or_default().push(Node {
            node_type: "TITLE".to_string(),
            value: title.text.clone(),
            row: title.row,
            col: title.col,
            divisions: 0,
            dash_consumed: false,
            nodes: Vec::new(),
        });
    }
    for directive in &document.metadata.directives {
        let key_node = Node {
            node_type: "DIRECTIVE_KEY".to_string(),
            value: format!("{}:", directive.key),
            row: directive.row,
            col: directive.col,
            divisions: 0,
            dash_consumed: false,
            nodes: Vec::new(),
        };
        let value_node = Node {
            node_type: "DIRECTIVE_VALUE".to_string(),
            value: directive.value.clone(),
            row: directive.row,
            col: directive.col + directive.key.len() + 1,
            divisions: 0,
            dash_consumed: false,
            nodes: Vec::new(),
        };
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


fn lex_text(raw_text: &str) -> Vec<LineInfo> {
    let mut lines_data = Vec::new();
    let chunk_regex = Regex::new(r"\S+").unwrap();

    for (line_num, line_text) in raw_text.lines().enumerate() {
        let mut chunks = Vec::new();
        for mat in chunk_regex.find_iter(line_text) {
            chunks.push(ChunkInfo {
                value: mat.as_str().to_string(),
                col: mat.start(), // 0-based for internal use
            });
        }
        lines_data.push(LineInfo {
            line_number: line_num + 1,
            line_text: line_text.to_string(),
            chunks,
        });
    }
    lines_data
}

fn tokenize_chunk(chunk: &str, line_num: usize, col_num: usize) -> Vec<Token> {
    let pitch_regex = Regex::new(r"^([SrRgGmMPdDnN]#?|-)+$").unwrap();
    if pitch_regex.is_match(chunk) {
        let mut tokens = Vec::new();
        let pitch_finder_regex = Regex::new(r"[SrRgGmMPdDnN]#?|-").unwrap();
        for mat in pitch_finder_regex.find_iter(chunk) {
            tokens.push(Token {
                token_type: TokenType::Pitch.as_str().to_string(),
                value: mat.as_str().to_string(),
                line: line_num,
                col: col_num + mat.start(),
            });
        }
        return tokens;
    }
    if ["|", "||", "|.", ":|", "|:", ":|:"].contains(&chunk) {
        return vec![Token {
            token_type: TokenType::Barline.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    if chunk.chars().all(|c| c == '-') {
        return vec![Token {
            token_type: TokenType::Pitch.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    if chunk.chars().any(|c| c.is_alphabetic())
        || (chunk.chars().any(|c| c.is_numeric())
            && chunk.chars().any(|c| !c.is_alphanumeric()))
    {
        return vec![Token {
            token_type: TokenType::Word.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    if chunk.chars().all(|c| !c.is_alphanumeric()) {
        return vec![Token {
            token_type: TokenType::Symbols.as_str().to_string(),
            value: chunk.to_string(),
            line: line_num,
            col: col_num,
        }];
    }
    vec![Token {
        token_type: TokenType::Unknown.as_str().to_string(),
        value: chunk.to_string(),
        line: line_num,
        col: col_num,
    }]
}

fn parse_css_for_ansi(css_path: &str) -> HashMap<String, (String, bool)> {
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

fn colorize_string(s: &str, color: &str, reverse: bool) -> String {
    let mut colored_s = match color {
        "yellow" => s.yellow(),
        "white" => s.white(),
        "green" => s.green(),
        "darkcyan" => s.cyan(),
        "red" => s.red(),
        "magenta" => s.magenta(),
        "blue" => s.blue(),
        "brown" => s.truecolor(165, 42, 42),
        _ => s.normal(),
    };
    if reverse {
        colored_s = colored_s.on_truecolor(50, 50, 50); // Dark grey background for reverse
    }
    colored_s.to_string()
}

fn generate_legend_string(
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let raw_text = fs::read_to_string(&args.input_file)?;
    let lines_info = lex_text(&raw_text);

    // --- Stage 1: Lexer ---
    let mut all_tokens: Vec<Token> = Vec::new();
    for line in &lines_info {
        let mut current_pos = 0;
        for chunk in &line.chunks {
            if chunk.col > current_pos {
                all_tokens.push(Token {
                    token_type: TokenType::Whitespace.as_str().to_string(),
                    value: " ".repeat(chunk.col - current_pos),
                    line: line.line_number,
                    col: current_pos,
                });
            }
            all_tokens.extend(tokenize_chunk(&chunk.value, line.line_number, chunk.col));
            current_pos = chunk.col + chunk.value.len();
        }
        if current_pos < line.line_text.len() {
            all_tokens.push(Token {
                token_type: TokenType::Whitespace.as_str().to_string(),
                value: " ".repeat(line.line_text.len() - current_pos),
                line: line.line_number,
                col: current_pos,
            });
        }
        all_tokens.push(Token {
            token_type: "NEWLINE".to_string(),
            value: "\n".to_string(),
            line: line.line_number,
            col: line.line_text.len(),
        });
    }

    // --- Used Tokens ---
    let mut used_tokens: HashMap<String, String> = HashMap::new();
    for token in &all_tokens {
        used_tokens
            .entry(token.token_type.clone())
            .or_insert_with(|| token.value.clone());
    }

    // --- Lexer Artifacts ---
    control::set_override(true);
    let styles = parse_css_for_ansi("styles.css");
    let legend_string_tokenizer = generate_legend_string(&styles, &used_tokens, None, false);

    // Build the raw tokenizer output first to calculate its width
    let mut raw_tokenizer_output = String::new();
    for token in &all_tokens {
        let (color, reverse) = styles
            .get(&token.token_type)
            .cloned()
            .unwrap_or(("white".to_string(), false));
        raw_tokenizer_output.push_str(&colorize_string(&token.value, &color, reverse));
    }

    let title_text = "--- Tokenizer Output ---";
    let tokenizer_output = format!(
        "{}\n\n{}\n{}",
        legend_string_tokenizer,
        title_text.bold(),
        raw_tokenizer_output
    );

    // .clr file
    let tokenizer_clr_path = args.input_file.with_extension("tokenizer.clr");
    fs::write(&tokenizer_clr_path, &tokenizer_output)?;
    eprintln!("Wrote tokenizer output to {}", tokenizer_clr_path.display());
    // .json file
    let lexer_json_path = args.input_file.with_extension("lexer.json");
    fs::write(&lexer_json_path, serde_json::to_string_pretty(&all_tokens)?)?;
    eprintln!("Wrote lexer JSON output to {}", lexer_json_path.display());


    // --- Stage 1.5: Metadata Parser ---
    let (metadata, musical_tokens) = parse_metadata(&all_tokens);

    // --- Stage 2: Flatten Spatial Relationships ---
    let nodes = flatten_spatial_relationships(&musical_tokens, &lines_info);

    // --- Stage 3: Beat Grouping ---
    let lines_of_music: Vec<usize> = all_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();
    let structured_nodes = group_nodes_into_lines_and_beats(&nodes, &lines_of_music);

    // --- Final Document Assembly ---
    let document = Document {
        metadata,
        nodes: structured_nodes,
    };

    // --- Flatten Spatial Relationships Artifacts ---
    // .yaml file
    let flattener_yaml_path = args.input_file.with_extension("flattener.yaml");
    let yaml_output = serde_yaml::to_string(&document)?;
    fs::write(&flattener_yaml_path, &yaml_output)?;
    eprintln!("Wrote flattener YAML output to {}", flattener_yaml_path.display());

    // .clr file
    let main_lines: HashSet<usize> = all_tokens.iter().filter(|t| t.token_type == "BARLINE").map(|t| t.line).collect();
    let legend_string_flattener = generate_legend_string(&styles, &used_tokens, Some(&document.metadata), true);
    let flattened_spatial_output = format!(
        "{}\n\n{}",
        legend_string_flattener,
        generate_flattened_spatial_view(&document, &lines_info, &styles, &main_lines)
    );
    let flattened_spatial_clr_path = args.input_file.with_extension("flattener.clr");
    fs::write(&flattened_spatial_clr_path, &flattened_spatial_output)?;
    eprintln!("Wrote flatten spatial relationships output to {}", flattened_spatial_clr_path.display());

    // --- LilyPond Output ---
    let lilypond_output = convert_to_lilypond(&document);
    let lilypond_path = args.input_file.with_extension("ly");
    fs::write(&lilypond_path, &lilypond_output)?;
    eprintln!("Wrote LilyPond output to {}", lilypond_path.display());

    // --- Final Output to Stdout ---
    control::set_override(false); // Use terminal detection for stdout
    print!("{}", flattened_spatial_output);

    Ok(())
}