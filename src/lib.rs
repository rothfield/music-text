use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Import our existing modules
mod models;
pub mod models_v2; // New type-safe parser AST
mod lexer;
pub mod handwritten_lexer;
mod spatial_analysis;
mod node_builder;
mod region_processor;
mod region_processor_v2; // New region processor for ParsedElement
mod rhythm_fsm_v2; // New FSM for ParsedElement
mod lyrics_v2; // New lyrics processor for ParsedElement
mod pitch;
mod display;
mod lilypond_converter;
pub mod lilypond_converter_v2; // New V2 converter - no conversion needed!
mod lilypond_templates;
mod outline;
mod vexflow_fsm_converter;
mod rhythm;
pub mod rhythm_fsm;
mod notation_detector;
mod lyrics;

pub use models::*;
pub use lexer::{lex_text, tokenize_chunk};
pub use handwritten_lexer::tokenize_with_handwritten_lexer;
pub use lilypond_converter::{convert_to_lilypond, convert_to_lilypond_with_template};
pub use lilypond_templates::{LilyPondTemplate, TemplateContext};
pub use pitch::LilyPondNoteNames;
pub use display::generate_flattened_spatial_view;
pub use outline::{generate_outline, ToOutline};
pub use pitch::{lookup_pitch, guess_notation, Notation};
pub use vexflow_fsm_converter::{convert_fsm_to_vexflow, VexFlowStave, VexFlowElement};

/// Complete parsing result structure for WASM API
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ParseResult {
    success: bool,
    error_message: Option<String>,
    document: Option<Document>,
    lilypond_output: String,
    yaml_output: String,
    json_output: String,
    outline_output: String,
    legend: String,
    detected_system: String,
    vexflow_output: String,
    spatial_analysis_output: String,
}

#[wasm_bindgen]
impl ParseResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }
    
    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn lilypond_output(&self) -> String {
        self.lilypond_output.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn yaml_output(&self) -> String {
        self.yaml_output.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn json_output(&self) -> String {
        self.json_output.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn outline_output(&self) -> String {
        self.outline_output.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn legend(&self) -> String {
        self.legend.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn detected_system(&self) -> String {
        self.detected_system.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn vexflow_output(&self) -> String {
        self.vexflow_output.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn spatial_analysis_output(&self) -> String {
        self.spatial_analysis_output.clone()
    }
    
    /// Get English LilyPond output from the stored document
    pub fn get_english_lilypond_output(&self, source_text: &str) -> String {
        if let Some(ref doc) = self.document {
            match convert_to_lilypond(doc, LilyPondNoteNames::English, Some(source_text)) {
                Ok(lily) => lily,
                Err(e) => format!("Error: {}", e),
            }
        } else {
            "No document available".to_string()
        }
    }
    
    /// Get FSM VexFlow output from the stored document
    pub fn get_fsm_vexflow_output(&self) -> String {
        if let Some(ref doc) = self.document {
            match convert_fsm_to_vexflow(doc) {
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

/// Convert slur attributes on nodes to actual SLUR_START and SLUR_END tokens
fn convert_slur_attributes_to_tokens(nodes: &mut Vec<Node>) {
    for node in nodes.iter_mut() {
        // Recursively process child nodes first
        convert_slur_attributes_to_tokens(&mut node.nodes);
        
        // Then convert slur attributes to tokens in this node's children
        convert_slur_attributes_in_children(&mut node.nodes);
    }
}

fn convert_slur_attributes_in_children(children: &mut Vec<Node>) {
    let mut new_children = Vec::new();
    
    for mut child in children.drain(..) {
        // Check if this child needs a slur start token before it
        if child.slur_start == Some(true) {
            let slur_start_node = Node::new(
                "SLUR_START".to_string(),
                "(".to_string(),
                child.row,
                child.col,
            );
            new_children.push(slur_start_node);
            child.slur_start = None; // Clear the attribute
        }
        
        // Add the child itself
        new_children.push(child);
        
        // Check if the last child needs a slur end token after it
        if let Some(last_child) = new_children.last_mut() {
            if last_child.slur_end == Some(true) {
                let slur_end_node = Node::new(
                    "SLUR_END".to_string(),
                    ")".to_string(),
                    last_child.row,
                    last_child.col + 1,
                );
                last_child.slur_end = None; // Clear the attribute
                new_children.push(slur_end_node);
            }
        }
    }
    
    *children = new_children;
}


pub fn unified_parser(input_text: &str) -> Result<(Document, String), Box<dyn std::error::Error>> {
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
    // Phase 1: Attach floating elements (spatial analysis) with intelligent lyrics detection
    let (hierarchical_nodes, _consumed_coords) = node_builder::attach_floating_elements(&remaining_tokens, &lines_info, input_text);
    
    // Find lines that contain musical content for Phase 2
    let lines_of_music = find_musical_lines(&remaining_tokens);
    
    // Step 4.4: Apply beat bracket and slur attributes to hierarchical nodes before FSM processing
    let mut hierarchical_nodes_with_attrs = hierarchical_nodes.clone();
    region_processor::apply_slurs_and_regions_to_nodes(&mut hierarchical_nodes_with_attrs, &remaining_tokens);
    
    // Save spatial analysis output for debugging
    let spatial_analysis_yaml = serde_yaml::to_string(&hierarchical_nodes_with_attrs).unwrap_or_else(|e| format!("YAML serialization error: {}", e));

    // Phase 2: Group nodes into lines and beats using FSM (musical structuring)
    eprintln!("UNIFIED_PARSER: About to call FSM with {} hierarchical_nodes", hierarchical_nodes_with_attrs.len());
    let mut structured_nodes = rhythm_fsm::group_nodes_with_fsm(&hierarchical_nodes_with_attrs, &lines_of_music);

    // Step 4.6: Convert slur attributes to actual tokens
    convert_slur_attributes_to_tokens(&mut structured_nodes);
    
    // Step 4.7: Process lyrics if present
    if lyrics::has_lyrics(&remaining_tokens, &lines_of_music) {
        let lyrics_lines = lyrics::parse_lyrics_lines(&remaining_tokens, input_text);
        lyrics::distribute_syllables_to_notes(&mut structured_nodes, lyrics_lines);
    }

    // Step 5: Create document
    let document = Document {
        metadata: metadata.clone(),
        nodes: structured_nodes,
        notation_system: metadata.detected_system.clone(),
    };

    Ok((document, spatial_analysis_yaml))
}

/// V2 Parser using ParsedElement instead of Node
pub fn unified_parser_v2(input_text: &str) -> Result<(models_v2::DocumentV2, String), Box<dyn std::error::Error>> {
    // First detect the notation type
    let detected_notation = notation_detector::detect_notation_type(input_text);
    
    // Use handwritten lexer for tokenization
    let all_tokens = handwritten_lexer::tokenize_with_handwritten_lexer(input_text);
    
    // Create lines_info for spatial analysis compatibility
    let lines_info = lex_text(input_text);
    
    // Parse metadata and detect notation system
    let (mut metadata, remaining_tokens) = lexer::parse_metadata(&all_tokens);
    metadata.detected_system = Some(detected_notation.as_str().to_string());

    // Phase 1: Attach floating elements using new ParsedElement system
    let (mut elements, _consumed_coords) = node_builder::attach_floating_elements_v2(&remaining_tokens, &lines_info, input_text);
    
    // Phase 2: Apply slur regions and beat brackets
    region_processor_v2::apply_slurs_and_regions_to_elements(&mut elements, &remaining_tokens);
    
    // Save spatial analysis output for debugging
    let spatial_analysis_yaml = serde_yaml::to_string(&elements).unwrap_or_else(|e| format!("YAML serialization error: {}", e));

    // Phase 3: Group elements into lines and beats using FSM
    let lines_of_music = find_musical_lines(&remaining_tokens);
    let mut structured_elements = rhythm_fsm_v2::group_elements_with_fsm(&elements, &lines_of_music);
    
    // Phase 4: Process lyrics if present
    if lyrics::has_lyrics(&remaining_tokens, &lines_of_music) {
        let lyrics_lines = lyrics_v2::parse_lyrics_lines(&remaining_tokens, input_text);
        lyrics_v2::distribute_syllables_to_elements(&mut structured_elements, lyrics_lines);
    }

    // Create DocumentV2
    let document_v2 = models_v2::DocumentV2 {
        metadata: metadata.clone(),
        elements: structured_elements,
        notation_system: metadata.detected_system.clone(),
    };

    Ok((document_v2, spatial_analysis_yaml))
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
    
    // Check each line for musical content (2+ pitches)
    for (line_num, line_tokens) in tokens_by_line {
        if line_tokens.len() >= 2 {
            musical_lines.push(line_num);
        }
    }
    
    musical_lines.sort();
    musical_lines.dedup();
    musical_lines
}

// Configuration for the parser
static mut LILYPOND_DISABLED: bool = false;

#[wasm_bindgen]
pub fn parse_notation(input_text: &str) -> ParseResult {
    // Set panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    match parse_notation_internal(input_text) {
        Ok((lilypond, yaml, json, outline, legend, detected_system, document, vexflow, spatial_analysis_yaml)) => {
            ParseResult {
                success: true,
                error_message: None,
                document: Some(document),
                lilypond_output: lilypond,
                yaml_output: yaml,
                json_output: json,
                outline_output: outline,
                legend,
                detected_system,
                vexflow_output: vexflow,
                spatial_analysis_output: spatial_analysis_yaml,
            }
        }
        Err(e) => {
            // Store error message for get_error_message()
            let error_msg = format!("Parsing error: {}", e);
            LAST_ERROR_MESSAGE.with(|s| *s.borrow_mut() = error_msg.clone());
            
            ParseResult {
                success: false,
                error_message: Some(error_msg),
                document: None,
                lilypond_output: String::new(),
                yaml_output: String::new(),
                json_output: String::new(),
                outline_output: String::new(),
                legend: String::new(),
                detected_system: "???".to_string(),
                vexflow_output: String::new(),
                spatial_analysis_output: String::new(),
            }
        }
    }
}



/// Legacy function for compatibility - use parse_notation() instead
#[wasm_bindgen]
pub fn convert_lilypond_to_vexflow_json(_lilypond_code: &str) -> String {
    format!("{{\"error\": \"LilyPond-based VexFlow conversion no longer supported\"}}")
}


#[wasm_bindgen]
pub fn get_build_timestamp() -> String {
    // Include actual build timestamp using compile-time environment
    format!("Built on {} at {} (v{})", 
        option_env!("BUILD_DATE").unwrap_or("unknown"), 
        option_env!("BUILD_TIME").unwrap_or("unknown"), 
        env!("CARGO_PKG_VERSION"))
}

#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Store the last parse results for getter functions
// Using thread-locals for WASM since it's single-threaded
use std::cell::RefCell;

thread_local! {
    static LAST_DETECTED_SYSTEM: RefCell<String> = RefCell::new("Unknown".to_string());
    static LAST_LILYPOND_OUTPUT: RefCell<String> = RefCell::new(String::new());
    static LAST_COLORIZED_OUTPUT: RefCell<String> = RefCell::new(String::new());
    static LAST_OUTLINE_OUTPUT: RefCell<String> = RefCell::new(String::new());
    static LAST_YAML_OUTPUT: RefCell<String> = RefCell::new(String::new());
    static LAST_ERROR_MESSAGE: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen]
pub fn get_detected_system() -> String {
    LAST_DETECTED_SYSTEM.with(|s| s.borrow().clone())
}

#[wasm_bindgen]
pub fn get_lilypond_output() -> String {
    LAST_LILYPOND_OUTPUT.with(|s| s.borrow().clone())
}

#[wasm_bindgen]
pub fn get_colorized_output() -> String {
    LAST_COLORIZED_OUTPUT.with(|s| s.borrow().clone())
}

#[wasm_bindgen]
pub fn get_outline_output() -> String {
    LAST_OUTLINE_OUTPUT.with(|s| s.borrow().clone())
}

#[wasm_bindgen]
pub fn get_yaml_output() -> String {
    LAST_YAML_OUTPUT.with(|s| s.borrow().clone())
}

#[wasm_bindgen]
pub fn get_error_message() -> String {
    LAST_ERROR_MESSAGE.with(|s| s.borrow().clone())
}

fn parse_notation_internal(input_text: &str) -> Result<(String, String, String, String, String, String, Document, String, String), Box<dyn std::error::Error>> {
    // Use V2 parser with direct V2 LilyPond converter - no conversion needed!
    let (document_v2, spatial_analysis_yaml) = unified_parser_v2(input_text)?;
    let detected_system = document_v2.metadata.detected_system.clone().unwrap_or("???".to_string());
    
    // Convert V2 to legacy Document only for WASM compatibility (other outputs)
    let document: Document = document_v2.clone().into();
    
    // Store the detected system for get_detected_system()
    LAST_DETECTED_SYSTEM.with(|s| *s.borrow_mut() = detected_system.clone());
    
    // Clear error message on successful parse
    LAST_ERROR_MESSAGE.with(|s| *s.borrow_mut() = String::new());

    // Generate outputs
    
    let all_tokens = handwritten_lexer::tokenize_with_handwritten_lexer(input_text);

    let legend = generate_legend(&all_tokens, &document.metadata);
    
    // Store colorized output (legend) for get_colorized_output()
    LAST_COLORIZED_OUTPUT.with(|s| *s.borrow_mut() = legend.clone());
    
    let lilypond_output = unsafe {
        if LILYPOND_DISABLED {
            "% LilyPond output disabled".to_string()
        } else {
            // Use V2 converter directly - no conversion bugs!
            lilypond_converter_v2::convert_document_v2_to_lilypond(&document_v2, LilyPondNoteNames::English, Some(input_text))
                .unwrap_or_else(|e| format!("LilyPond V2 error: {}", e))
        }
    };
    
    // Store LilyPond output for get_lilypond_output()
    LAST_LILYPOND_OUTPUT.with(|s| *s.borrow_mut() = lilypond_output.clone());
    let yaml_output = serde_yaml::to_string(&document)?;
    let json_output = serde_json::to_string_pretty(&document)?;
    let outline_output = document.to_html_outline(0);
    
    // Store outline output for get_outline_output()
    LAST_OUTLINE_OUTPUT.with(|s| *s.borrow_mut() = outline_output.clone());
    
    // Store YAML output for get_yaml_output()
    LAST_YAML_OUTPUT.with(|s| *s.borrow_mut() = yaml_output.clone());
    
    // Debug: Check if outline contains what we expect
    if !outline_output.contains("musical-line") {
        // For web compatibility, ensure we have musical-line in the output
        // (This might happen if the document structure uses 'line' instead of 'musical-line')
        eprintln!("Warning: outline_output does not contain 'musical-line': {}", outline_output);
    }
    
    // Generate VexFlow output (now from FSM, not LilyPond)
    let vexflow_output = unsafe {
        if LILYPOND_DISABLED {
            // Use FSM-based VexFlow generation
            match vexflow_fsm_converter::convert_fsm_to_vexflow(&document) {
                Ok(vexflow_staves) => serde_json::to_string(&vexflow_staves).unwrap_or_else(|e| format!("{{\"error\": \"JSON serialization error: {}\"}}", e)),
                Err(e) => format!("{{\"error\": \"VexFlow FSM conversion error: {}\"}}", e)
            }
        } else {
            // Legacy LilyPond-based VexFlow generation removed
            format!("{{\"error\": \"LilyPond-based VexFlow conversion no longer supported\"}}")
        }
    };

    Ok((lilypond_output, yaml_output, json_output, outline_output, legend, detected_system, document, vexflow_output, spatial_analysis_yaml))
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