use wasm_bindgen::prelude::*;
use std::collections::HashMap;

// Import our existing modules
mod models;
mod lexer;
pub mod handwritten_lexer;
mod parser;
mod pitch;
mod colorizer;
mod display;
mod rhythmic_converter;
mod lilypond_converter;
mod outline;
pub mod formatter;
mod vexflow_converter;
mod vexflow_fsm_converter;
mod rhythm;
pub mod rhythm_fsm;
mod notation_detector;

pub use models::*;
pub use lexer::{lex_text, tokenize_chunk};
pub use handwritten_lexer::tokenize_with_handwritten_lexer;
pub use lilypond_converter::{convert_to_lilypond, convert_to_lilypond_with_names};
pub use rhythmic_converter::LilyPondNoteNames;
pub use display::generate_flattened_spatial_view;
pub use colorizer::{parse_css_for_ansi, colorize_string, generate_legend_string};
pub use outline::{generate_outline, ToOutline};
pub use pitch::{lookup_pitch, guess_notation, Notation};
pub use formatter::format_document_to_text;
pub use vexflow_converter::{LilyPondToVexFlowConverter, VexFlowNote};
pub use vexflow_fsm_converter::{convert_fsm_to_vexflow, VexFlowStave, VexFlowElement};

pub fn unified_parser(input_text: &str) -> Result<Document, Box<dyn std::error::Error>> {
    // First detect the notation type (this will be used by the lexer)
    let detected_notation = notation_detector::detect_notation_type(input_text);
    
    // Use handwritten lexer for tokenization (which now uses the detected notation)
    let all_tokens = handwritten_lexer::tokenize_with_handwritten_lexer(input_text);
    
    // Create lines_info for spatial analysis (still needed for parser phases)
    let lines_info = lex_text(input_text);
    
    // Step 3: Parse metadata and detect notation system
    let (mut metadata, remaining_tokens) = lexer::parse_metadata(&all_tokens);

    // Store the detected notation system
    metadata.detected_system = Some(detected_notation.as_str().to_string());

    // Step 4: Use the complete parsing pipeline
    // Phase 1: Attach floating elements (spatial analysis)
    let hierarchical_nodes = parser::attach_floating_elements(&remaining_tokens, &lines_info);
    
    // Find lines that contain musical content for Phase 2
    let lines_of_music = find_musical_lines(&remaining_tokens);
    
    // Phase 2: Group nodes into lines and beats using FSM (musical structuring)
    eprintln!("UNIFIED_PARSER: About to call FSM with {} hierarchical_nodes", hierarchical_nodes.len());
    let structured_nodes = rhythm_fsm::group_nodes_with_fsm(&hierarchical_nodes, &lines_of_music);

    // Step 5: Create document
    let document = Document {
        metadata: metadata.clone(),
        nodes: structured_nodes,
        notation_system: metadata.detected_system.clone(),
    };

    Ok(document)
}

fn find_musical_lines(tokens: &[Token]) -> Vec<usize> {
    let mut musical_lines = Vec::new();
    let mut tokens_by_line: std::collections::HashMap<usize, Vec<&Token>> = std::collections::HashMap::new();
    
    // Group tokens by line
    for token in tokens {
        if token.token_type == "PITCH" {
            tokens_by_line.entry(token.line).or_default().push(token);
        }
    }
    
    // Check each line for musical content (3+ pitches)
    for (line_num, line_tokens) in tokens_by_line {
        if line_tokens.len() >= 3 {
            musical_lines.push(line_num);
        }
    }
    
    musical_lines.sort();
    musical_lines.dedup();
    musical_lines
}

// Global result storage for WASM
static mut LAST_COLORIZED_OUTPUT: Option<String> = None;
static mut LAST_LILYPOND_OUTPUT: Option<String> = None;
static mut LAST_YAML_OUTPUT: Option<String> = None;
static mut LAST_JSON_OUTPUT: Option<String> = None;
static mut LAST_OUTLINE_OUTPUT: Option<String> = None;
static mut LAST_ERROR_MESSAGE: Option<String> = None;
static mut LAST_LEGEND: Option<String> = None;
static mut LAST_DETECTED_SYSTEM: Option<String> = None;
static mut LAST_DOCUMENT: Option<Document> = None;
static mut LAST_VEXFLOW_OUTPUT: Option<String> = None;

#[wasm_bindgen]
pub fn parse_notation(input_text: &str) -> bool {
    // Set panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    match parse_notation_internal(input_text) {
        Ok((colorized, lilypond, yaml, json, outline, legend, detected_system, document, vexflow)) => {
            unsafe {
                LAST_COLORIZED_OUTPUT = Some(colorized);
                LAST_LILYPOND_OUTPUT = Some(lilypond);
                LAST_YAML_OUTPUT = Some(yaml);
                LAST_JSON_OUTPUT = Some(json);
                LAST_OUTLINE_OUTPUT = Some(outline);
                LAST_LEGEND = Some(legend);
                LAST_DETECTED_SYSTEM = Some(detected_system);
                LAST_DOCUMENT = Some(document);
                LAST_VEXFLOW_OUTPUT = Some(vexflow);
                LAST_ERROR_MESSAGE = None;
            }
            true
        }
        Err(e) => {
            unsafe {
                LAST_COLORIZED_OUTPUT = None;
                LAST_LILYPOND_OUTPUT = None;
                LAST_YAML_OUTPUT = None;
                LAST_JSON_OUTPUT = None;
                LAST_OUTLINE_OUTPUT = None;
                LAST_LEGEND = None;
                LAST_DETECTED_SYSTEM = None;
                LAST_VEXFLOW_OUTPUT = None;
                // On error, we do not clear the last good document
                LAST_ERROR_MESSAGE = Some(format!("Parsing error: {}", e));
            }
            false
        }
    }
}

#[wasm_bindgen]
pub fn get_colorized_output() -> String {
    unsafe { LAST_COLORIZED_OUTPUT.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_lilypond_output() -> String {
    unsafe { LAST_LILYPOND_OUTPUT.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_yaml_output() -> String {
    unsafe { LAST_YAML_OUTPUT.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_json_output() -> String {
    unsafe { LAST_JSON_OUTPUT.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_outline_output() -> String {
    unsafe { LAST_OUTLINE_OUTPUT.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_error_message() -> String {
    unsafe { LAST_ERROR_MESSAGE.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_legend() -> String {
    unsafe { LAST_LEGEND.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_detected_system() -> String {
    unsafe { LAST_DETECTED_SYSTEM.as_ref().cloned().unwrap_or("???".to_string()) }
}

#[wasm_bindgen]
pub fn get_formatted_text() -> String {
    unsafe {
        if let Some(doc) = LAST_DOCUMENT.as_ref() {
            formatter::format_document_to_text(doc)
        } else {
            String::new()
        }
    }
}

#[wasm_bindgen]
pub fn get_vexflow_output() -> String {
    unsafe { LAST_VEXFLOW_OUTPUT.as_ref().cloned().unwrap_or_default() }
}

#[wasm_bindgen]
pub fn get_fsm_vexflow_output() -> String {
    unsafe {
        if let Some(doc) = LAST_DOCUMENT.as_ref() {
            match vexflow_fsm_converter::convert_fsm_to_vexflow(doc) {
                Ok(staves) => {
                    match serde_json::to_string(&staves) {
                        Ok(json) => json,
                        Err(e) => format!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
                    }
                }
                Err(e) => format!("{{\"error\": \"VexFlow conversion failed: {}\"}}", e)
            }
        } else {
            "[]".to_string()
        }
    }
}

#[wasm_bindgen]
pub fn convert_lilypond_to_vexflow_json(lilypond_code: &str) -> String {
    match vexflow_converter::LilyPondToVexFlowConverter::new() {
        Ok(converter) => {
            match converter.convert_lilypond_to_vexflow(lilypond_code) {
                Ok(notes) => {
                    match converter.to_vexflow_json(&notes) {
                        Ok(json) => json,
                        Err(e) => format!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
                    }
                }
                Err(e) => format!("{{\"error\": \"Conversion failed: {}\"}}", e)
            }
        }
        Err(e) => format!("{{\"error\": \"Converter initialization failed: {}\"}}", e)
    }
}

#[wasm_bindgen]
pub fn get_english_lilypond_output() -> String {
    unsafe {
        if let Some(doc) = LAST_DOCUMENT.as_ref() {
            match convert_to_lilypond_with_names(doc, LilyPondNoteNames::English) {
                Ok(lily) => lily,
                Err(e) => format!("Error: {}", e),
            }
        } else {
            "No document available".to_string()
        }
    }
}

#[wasm_bindgen]
pub fn get_build_timestamp() -> String {
    format!("{} built Aug 2 19:20 (triplet detection + proper beaming)", env!("CARGO_PKG_VERSION"))
}

#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}


fn parse_notation_internal(input_text: &str) -> Result<(String, String, String, String, String, String, String, Document, String), Box<dyn std::error::Error>> {
    let document = unified_parser(input_text)?;
    let detected_system = document.metadata.detected_system.clone().unwrap_or("???".to_string());

    // Generate outputs  
    let main_lines_set: std::collections::HashSet<usize> = document.nodes
        .iter()
        .filter(|n| n.node_type == "LINE" && n.nodes.iter().any(|child| 
            child.node_type == "PITCH" || child.node_type == "BARLINE"
        ))
        .map(|n| n.row)
        .collect();
    
    let all_tokens = handwritten_lexer::tokenize_with_handwritten_lexer(input_text);

    let legend = generate_legend(&all_tokens, &document.metadata);
    let final_parse_output = generate_css_styled_html(&document, &lex_text(input_text), &main_lines_set);
    let colorized_output = format!("{}\n<strong>--- Final Parse Output ---</strong><br>\n{}", legend, final_parse_output);
    let lilypond_output = convert_to_lilypond(&document).unwrap_or_else(|e| format!("LilyPond error: {}", e));
    let yaml_output = serde_yaml::to_string(&document)?;
    let json_output = serde_json::to_string_pretty(&document)?;
    let outline_output = document.to_html_outline(0);
    
    // Generate VexFlow output from LilyPond
    let vexflow_output = match vexflow_converter::LilyPondToVexFlowConverter::new() {
        Ok(converter) => {
            match converter.convert_lilypond_to_vexflow(&lilypond_output) {
                Ok(notes) => converter.to_vexflow_json(&notes).unwrap_or_else(|e| format!("{{\"error\": \"VexFlow JSON error: {}\"}}", e)),
                Err(e) => format!("{{\"error\": \"VexFlow conversion error: {}\"}}", e)
            }
        }
        Err(e) => format!("{{\"error\": \"VexFlow converter init error: {}\"}}", e)
    };

    Ok((colorized_output, lilypond_output, yaml_output, json_output, outline_output, legend, detected_system, document, vexflow_output))
}

// Generate HTML with CSS classes instead of ANSI codes
fn generate_css_styled_html(
    document: &Document,
    lines_info: &[crate::models::LineInfo],
    _main_lines: &std::collections::HashSet<usize>,
) -> String {
    let mut output_lines = Vec::new();
    
    // Create metadata nodes for lines that need them
    let mut metadata_by_line: HashMap<usize, Vec<crate::models::Node>> = HashMap::new();
    if let Some(title) = &document.metadata.title {
        metadata_by_line.entry(title.row).or_default().push(crate::models::Node::new(
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
        
        let key_node = crate::models::Node::new(
            "DIRECTIVE_KEY".to_string(),
            format!("{}:", directive.key),
            directive.row,  
            directive.col,
        );
        let value_node = crate::models::Node::new(
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
            // Collect ALL nodes that belong to this line, including WHITESPACE
            for node in &line_node.nodes {
                if node.row == line_info.line_number {
                    line_nodes.push(node.clone());
                }
                // Also collect beat content  
                if node.node_type == "BEAT" {
                    collect_beat_content_for_line(&node.nodes, &mut line_nodes, true, line_info.line_number);
                }
            }
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

            let (display_value, css_class) = if node.value.starts_with("BEAT_ELEMENT:") {
                let val = node.value.strip_prefix("BEAT_ELEMENT:").unwrap_or(&node.value);
                (val, format!("token-{} beat-element", node.node_type.to_lowercase().replace("_", "-")))
            } else {
                (node.value.as_str(), format!("token-{}", node.node_type.to_lowercase().replace("_", "-")))
            };
            
            if node.node_type == "TITLE" {
                line_output.push_str(&format!("<span class=\"token-title\">{}</span>", display_value));
            } else if node.node_type == "OCTAVE_MARKER" {
                // Special styling for attached octave markers: purple + italic to show attachment
                line_output.push_str(&format!("<span class=\"token-octave-marker\" style=\"color: purple; font-style: italic;\">{}</span>", display_value));
            } else if node.value.starts_with("BEAT_ELEMENT:") && node.node_type == "PITCH" {
                // Special styling for pitches inside beats: underline + italic
                line_output.push_str(&format!("<span class=\"{}\" style=\"text-decoration: underline; font-style: italic;\">{}</span>", css_class, display_value));
            } else {
                line_output.push_str(&format!("<span class=\"{}\">{}</span>", css_class, display_value));
            }
            
            current_col = node.col + display_value.len();
        }
        
        if current_col < line_info.line_text.len() {
            line_output.push_str(&" ".repeat(line_info.line_text.len() - current_col));
        }

        output_lines.push(line_output);
    }

    output_lines.join("<br>\n")
}

// Helper functions similar to display.rs

fn collect_beat_content_for_line(nodes: &[crate::models::Node], result: &mut Vec<crate::models::Node>, is_beat_element: bool, target_line: usize) {
    for node in nodes {
        if node.node_type != "NEWLINE" && node.row == target_line {
            let mut beat_node = node.clone();
            if is_beat_element {
                beat_node.value = format!("BEAT_ELEMENT:{}", beat_node.value);
            }
            result.push(beat_node);
            collect_beat_content_for_line(&node.nodes, result, is_beat_element, target_line);
        }
    }
}

fn collect_child_nodes_for_line(nodes: &[crate::models::Node], result: &mut Vec<crate::models::Node>, target_line: usize) {
    for node in nodes {
        if node.node_type == "BEAT" {
            collect_child_nodes_for_line(&node.nodes, result, target_line);
        } else {
            collect_child_nodes_for_line(&node.nodes, result, target_line);
            for child in &node.nodes {
                if child.row == target_line && child.node_type != "NEWLINE" {
                    result.push(child.clone());
                }
            }
        }
    }
}

// Generate legend HTML
fn generate_legend(tokens: &[crate::models::Token], metadata: &crate::models::Metadata) -> String {
    let mut used_tokens: HashMap<String, String> = HashMap::new();
    
    // Collect sample values for each token type
    for token in tokens {
        if !used_tokens.contains_key(&token.token_type) {
            used_tokens.insert(token.token_type.clone(), token.value.clone());
        }
    }
    
    let mut legend = String::new();
    legend.push_str("<div class=\"legend\">");
    legend.push_str("<strong>--- Active Token Legend ---</strong><br>");
    
    // Sort token types for consistent display
    let mut sorted_tokens: Vec<_> = used_tokens.iter().collect();
    sorted_tokens.sort_by_key(|(k, _)| *k);
    
    for (token_type, sample_value) in sorted_tokens {
        let css_class = format!("token-{}", token_type.to_lowercase().replace("_", "-"));
        legend.push_str(&format!(
            "- {}: <span class=\"{}\">{}</span><br>",
            token_type, css_class, sample_value
        ));
    }
    
    // Add metadata legend entries if present
    if metadata.title.is_some() {
        legend.push_str("- TITLE: <span class=\"token-title\">Title Text</span><br>");
    }
    if !metadata.directives.is_empty() {
        legend.push_str("- DIRECTIVE_KEY: <span class=\"token-directive-key\">key:</span><br>");
        legend.push_str("- DIRECTIVE_VALUE: <span class=\"token-directive-value\">value</span><br>");
    }
    
    legend.push_str("- OCTAVE_MARKER: <span class=\"token-octave-marker\" style=\"color: purple; font-style: italic;\">.':,</span><br>");
    legend.push_str("- UNASSIGNED: <span class=\"token-unassigned\"> </span><br>");
    legend.push_str("<strong>---------------------------</strong>");
    legend.push_str("</div>");
    
    legend
}

// Initialize function called when WASM module loads
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}