// Public API for pest grammar music-text parser
use serde::{Deserialize, Serialize};

// Internal modules
pub mod parser;
pub mod web_server;
mod ast;
mod ast_to_parsed;
mod rhythm_fsm;  
mod renderers;  
mod models;
mod parser_v2_fsm;
mod spatial_parser;
mod structure_preserving_fsm;

// Re-export key types for external use
pub use ast::{Document, NotationSystem};

/// Detect the predominant notation system from a parsed document
fn detect_notation_system(document: &Document) -> String {
    let mut system_counts = std::collections::HashMap::new();
    
    // Count pitch types across all staves
    for stave in &document.staves {
        for measure in &stave.content_line.measures {
            for beat in &measure.beats {
                for element in &beat.elements {
                    if let ast::BeatElement::Pitch { value, .. } = element {
                        let system = classify_pitch(value);
                        *system_counts.entry(system).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    
    // Return the most common system, or "auto" if none detected
    system_counts.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(system, _)| system)
        .unwrap_or_else(|| "auto".to_string())
}

/// Classify a pitch string to determine its notation system
fn classify_pitch(pitch: &str) -> String {
    let first_char = pitch.chars().next().unwrap_or('\0');
    
    match first_char {
        '1'..='7' => "number".to_string(),
        'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' | 's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' => "sargam".to_string(),
        'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => "western".to_string(),
        'c' | 'e' | 'f' | 'a' | 'b' => "abc".to_string(), // Distinguish from western by case
        _ => {
            // Check for doremi patterns
            if pitch.starts_with('d') || pitch.starts_with('r') || pitch.starts_with('m') || 
               pitch.starts_with('f') || pitch.starts_with('s') || pitch.starts_with('l') || 
               pitch.starts_with('t') {
                // Could be doremi or sargam - need more context
                if first_char.is_lowercase() {
                    "doremi".to_string()
                } else {
                    "sargam".to_string()
                }
            } else {
                "auto".to_string()
            }
        }
    }
}

/// Clean result structure for all parsing operations
#[derive(Debug, Serialize, Deserialize)]
pub struct NotationResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub document: Option<Document>,  // Final processed AST
    pub ast: Option<String>,         // Raw AST (before spatial processing)
    pub spatial: Option<String>,     // Spatial processed AST  
    pub fsm: Option<String>,         // FSM rhythm analysis output
    pub yaml: Option<String>,        // Human-readable format
    pub vexflow: Option<String>,     // Temporarily simplified  
    pub lilypond: Option<String>,    // LilyPond source
}

/// Main public API function - parse notation and return all formats
pub fn parse_notation_full(input: &str, system: Option<&str>) -> NotationResult {
    let system = system.unwrap_or("auto");
    
    match parser::parse_notation_with_stages(input, system) {
        Ok((raw_document, spatial_document)) => {
            
            // Use structure-preserving FSM approach
            let processed_doc = structure_preserving_fsm::ProcessedDocument::from_document(&spatial_document);
            
            // Keep spatial document for backward compatibility
            let enriched_document = spatial_document;
            
            // Generate raw AST as JSON
            let ast_json = match serde_json::to_string_pretty(&raw_document) {
                Ok(json_str) => Some(json_str),
                Err(_) => None,
            };
            
            // Generate spatial AST as JSON  
            let spatial_json = match serde_json::to_string_pretty(&enriched_document) {
                Ok(json_str) => Some(json_str),
                Err(_) => None,
            };
            
            // Generate FSM output as JSON (structure-preserving)
            let fsm_json = match serde_json::to_string_pretty(&processed_doc) {
                Ok(json_str) => Some(json_str),
                Err(_) => None,
            };
            
            // Generate final YAML representation (keep for compatibility)
            let yaml_output = match serde_yaml::to_string(&enriched_document) {
                Ok(yaml_str) => Some(format_yaml_with_indent(yaml_str)),
                Err(_) => None,
            };
            
            // Generate VexFlow output (temporarily disabled)
            let vexflow_output = None; // TODO: Fix VexFlow renderer compilation errors and add structure-preserving support
            
            // Generate LilyPond output using structure-preserving FSM data
            let lilypond_output = {
                let metadata = models::Metadata {
                    title: None,
                    directives: Vec::new(),
                    detected_system: None,
                    attributes: std::collections::HashMap::new(),
                };
                renderers::lilypond::renderer::convert_processed_document_to_lilypond_src(&processed_doc, &metadata, Some(input))
                    .ok()
            };
            
            NotationResult {
                success: true,
                error_message: None,
                document: Some(enriched_document),
                ast: ast_json,
                spatial: spatial_json,
                fsm: fsm_json,
                yaml: yaml_output,
                vexflow: vexflow_output,
                lilypond: lilypond_output,
            }
        }
        Err(e) => {
            NotationResult {
                success: false,
                error_message: Some(e.to_string()),
                document: None,
                ast: None,
                spatial: None,
                fsm: None,
                yaml: None,
                vexflow: None,
                lilypond: None,
            }
        }
    }
}


// Helper function to format YAML with proper indentation
fn format_yaml_with_indent(yaml_str: String) -> String {
    let mut result = String::from("---\n");
    for line in yaml_str.lines() {
        if line.starts_with("---") {
            continue;
        }
        if !line.is_empty() {
            let indent_level = line.len() - line.trim_start().len();
            let spaces_per_indent = 2;
            let new_indent_level = (indent_level / spaces_per_indent) * 4;
            result.push_str(&" ".repeat(new_indent_level));
            result.push_str(line.trim_start());
        }
        result.push('\n');
    }
    result
}