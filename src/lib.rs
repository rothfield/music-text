use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
pub use models::Document;

// Import our existing modules
mod models;
mod parser;
mod display;
pub mod converters; // Unified converter modules (LilyPond, VexFlow, shared utilities)
mod outline;
// mod lyrics; // DELETED - replaced with lyrics

pub use models::*;
pub use parser::lex_text; // tokenize_chunk DELETED - unused
pub use parser::tokenize_with_handwritten_lexer;
pub use parser::horizontal::*;
// pub use lilypond_converter::{convert_to_lilypond, convert_to_lilypond_with_template}; // DELETED - V1 converter unused
// pub use lilypond_templates::{LilyPondTemplate, TemplateContext}; // DELETED - V1 templates unused
pub use models::LilyPondNoteNames;
pub use display::generate_flattened_spatial_view;
pub use outline::{generate_outline, ToOutline};
pub use models::{lookup_pitch, Notation}; // guess_notation DELETED - unused
pub use converters::vexflow::{convert_elements_to_staff_notation, convert_elements_to_vexflow_js, StaffNotationStave, StaffNotationElement, StaffNotationAccidental};

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
}

impl ParseResult {
    // get_english_lilypond_output DELETED - unused V1 function
    
    /// Get the parsed document
    pub fn get_document(&self) -> Option<Document> {
        self.document.clone()
    }
    
    
}


// unified_parser V1 deleted - use unified_parser instead

/// Simple conversion from tokens to ParsedElement (replacement for node_builder)
fn convert_tokens_to_parsed_elements(tokens: &[Token], global_notation: crate::models::Notation) -> Vec<models::ParsedElement> {
    use crate::models::{ParsedElement, Position};
    use crate::models::{lookup_pitch};
    
    let mut elements = Vec::new();
    
    for token in tokens {
        let position = Position { row: token.line, col: token.col };
        
        match token.token_type.as_str() {
            "PITCH" => {
                // Use global notation detection instead of per-token guessing
                if let Some(degree) = lookup_pitch(&token.value, global_notation) {
                    elements.push(ParsedElement::Note {
                        degree,
                        octave: 0, // Default octave, will be calculated later
                        value: token.value.clone(),
                        position,
                        children: Vec::new(),
                        duration: None,
                        slur: None, // Will be set by vertical_parser
                    });
                } else {
                    elements.push(ParsedElement::Unknown {
                        value: token.value.clone(),
                        position,
                    });
                }
            },
            "BARLINE" => {
                elements.push(ParsedElement::Barline {
                    style: token.value.clone(),
                    position,
                    tala: None, // Will be set by vertical_parser
                });
            },
            "REST" => {
                elements.push(ParsedElement::Rest {
                    value: token.value.clone(),
                    position,
                    duration: None,
                });
            },
            "DASH" => {
                elements.push(ParsedElement::Dash {
                    degree: None, // Will be inherited later
                    octave: None,
                    position,
                    duration: None,
                });
            },
            "WHITESPACE" => {
                elements.push(ParsedElement::Whitespace {
                    width: token.value.len(),
                    position,
                });
            },
            "NEWLINE" => {
                elements.push(ParsedElement::Newline {
                    position,
                });
            },
            "WORD" => {
                elements.push(ParsedElement::Word {
                    text: token.value.clone(),
                    position,
                });
            },
            // "TALA" token type removed - numbers handled via context in vertical parser
            "SLUR_START" => {
                elements.push(ParsedElement::SlurStart {
                    position,
                });
            },
            "SLUR_END" => {
                elements.push(ParsedElement::SlurEnd {
                    position,
                });
            },
            // All other tokens become generic symbols
            _ => {
                elements.push(ParsedElement::Symbol {
                    value: token.value.clone(),
                    position,
                });
            }
        }
    }
    
    elements
}

/// Parse a key string to a degree for transposition
fn parse_key_to_degree(key_str: &str) -> Option<crate::models::Degree> {
    use crate::models::Degree;
    match key_str.to_uppercase().as_str() {
        "C" => Some(Degree::N1),
        "D" => Some(Degree::N2), 
        "E" => Some(Degree::N3),
        "F" => Some(Degree::N4),
        "G" => Some(Degree::N5),
        "A" => Some(Degree::N6),
        "B" => Some(Degree::N7),
        _ => None,
    }
}

/// V2 Parser using ParsedElement instead of Node
pub fn unified_parser(input_text: &str) -> Result<(models::ParsedDocument, String), Box<dyn std::error::Error>> {
    // First detect the notation type
    let detected_notation = parser::notation_detector::detect_notation_type(input_text);
    
    // Use handwritten lexer for tokenization
    let all_tokens = parser::tokenize_with_handwritten_lexer(input_text);
    
    // Create lines for spatial analysis compatibility
    let _lines = lex_text(input_text);
    
    // Parse metadata and detect notation system
    let (mut metadata, remaining_tokens) = parser::parse_metadata(&all_tokens);
    metadata.detected_system = Some(detected_notation.as_str().to_string());

    // Convert NotationType to Notation for pitch lookup
    let global_notation = match detected_notation {
        parser::NotationType::Western => crate::models::Notation::Western,
        parser::NotationType::Sargam => crate::models::Notation::Sargam,
        parser::NotationType::Number => crate::models::Notation::Number,
    };

    // Phase 1: Convert tokens to ParsedElement system using global notation detection
    let mut elements = convert_tokens_to_parsed_elements(&remaining_tokens, global_notation);
    
    // Phase 2: Apply slur regions and beat brackets
    parser::vertical::apply_slurs_and_regions_to_elements(&mut elements, &remaining_tokens);
    
    // Save spatial analysis output for debugging
    let spatial_analysis_yaml = serde_yaml::to_string(&elements).unwrap_or_else(|e| format!("YAML serialization error: {}", e));

    // Phase 3: Group elements into lines and beats using FSM
    let lines_of_music = find_musical_lines(&remaining_tokens);
    
    let mut elements = parser::horizontal::group_elements_with_fsm_full(&elements, &lines_of_music);
        
        // Check for Key in metadata and inject Tonic item at the beginning
        eprintln!("DEBUG: Metadata attributes: {:?}", metadata.attributes);
        
        // Try both "Key" and "key" for case-insensitive match
        let key_str = metadata.attributes.get("Key")
            .or_else(|| metadata.attributes.get("key"));
            
        if let Some(key_str) = key_str {
            eprintln!("DEBUG: Found Key in metadata: {}", key_str);
            // Parse the key string to get the tonic degree
            if let Some(tonic_degree) = parse_key_to_degree(key_str) {
                eprintln!("DEBUG: Parsed key '{}' to degree {:?}", key_str, tonic_degree);
                // Insert Tonic item at the beginning
                elements.insert(0, parser::horizontal::Item::Tonic(tonic_degree));
                eprintln!("DEBUG: Inserted Tonic item at beginning of elements");
            } else {
                eprintln!("DEBUG: Failed to parse key '{}' to degree", key_str);
            }
        } else {
            eprintln!("DEBUG: No Key found in metadata");
        }
        
    let mut structured_elements = parser::horizontal::convert_elements_to_elements_public(elements.clone());
    
    // Store FSM output for LilyPond conversion
    LAST_FSM_OUTPUT.with(|s| *s.borrow_mut() = elements);
    
    // Phase 4: Process lyrics if present
    if models::has_lyrics(&remaining_tokens, &lines_of_music) {
        let lyrics_lines = models::parse_lyrics_lines(&remaining_tokens, input_text);
        models::distribute_syllables_to_elements(&mut structured_elements, lyrics_lines);
    }

    // Create ParsedDocument
    let document_v2 = models::ParsedDocument {
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


#[wasm_bindgen]
pub fn parse_notation(input_text: &str) -> ParseResult {
    // Set panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    // V1 parse_notation deprecated - use V2 system instead
    match unified_parser(input_text) {
        Ok((document_v2, spatial_analysis_yaml)) => {
            let document: Document = document_v2.into();
            
            // Generate VexFlow JavaScript code using V2 converter
            let fsm_elements = get_last_elements();
            eprintln!("WASM DEBUG: FSM elements for VexFlow: {} items", fsm_elements.len());
            for (i, elem) in fsm_elements.iter().enumerate() {
                eprintln!("WASM DEBUG: Element {}: {:?}", i, elem);
            }
            let vexflow_js = match convert_elements_to_vexflow_js(&fsm_elements, &document.metadata) {
                Ok(js_code) => {
                    // Debug: Log first few lines of generated JS to verify it's the 2-pass generator
                    let first_lines: Vec<&str> = js_code.lines().take(5).collect();
                    eprintln!("WASM DEBUG: Generated VexFlow JS (first 5 lines):");
                    for line in &first_lines {
                        eprintln!("WASM DEBUG: {}", line);
                    }
                    js_code
                },
                Err(e) => {
                    eprintln!("WASM DEBUG: VexFlow JS generation failed: {}", e);
                    String::new()
                },
            };
            
            // Generate LilyPond output using V2 converter
            let lilypond_output = match converters::lilypond::convert_elements_to_lilypond_src(
                &get_last_elements(),
                &document.metadata,
                Some(input_text)
            ) {
                Ok(ly_content) => ly_content,
                Err(_) => String::new(),
            };
            
            ParseResult {
                success: true,
                error_message: None,
                document: Some(document),
                lilypond_output: lilypond_output,
                yaml_output: String::new(),
                json_output: String::new(),
                outline_output: String::new(),
                legend: String::new(),
                detected_system: "V2".to_string(),
                vexflow_output: vexflow_js,
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
    static LAST_FSM_OUTPUT: RefCell<Vec<parser::horizontal::Item>> = RefCell::new(Vec::new());
    static LAST_ERROR_MESSAGE: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen]
pub fn get_detected_system() -> String {
    LAST_DETECTED_SYSTEM.with(|s| s.borrow().clone())
}

/// Get the last FSM output for CLI use (avoid running FSM twice)
pub fn get_last_elements() -> Vec<parser::horizontal::Item> {
    LAST_FSM_OUTPUT.with(|s| s.borrow().clone())
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

/// Toggle slurs on selected text in a textarea
/// Returns modified text with slurs added/removed/toggled
#[wasm_bindgen]
pub fn toggle_slurs(text: &str, selection_start: usize, selection_end: usize) -> String {
    console_error_panic_hook::set_once();
    
    // Split text into lines
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }
    
    // Find which line contains the selection
    let mut char_count = 0;
    let mut target_line_idx = None;
    let mut line_start_pos = 0;
    
    for (line_idx, line) in lines.iter().enumerate() {
        let line_end = char_count + line.len();
        
        if selection_start >= char_count && selection_start <= line_end {
            target_line_idx = Some(line_idx);
            line_start_pos = char_count;
            break;
        }
        
        char_count = line_end + 1; // +1 for newline
    }
    
    let target_line_idx = match target_line_idx {
        Some(idx) => idx,
        None => return text.to_string(), // Selection not found
    };
    
    // Check if target line is a musical line (contains pitches)
    if !is_musical_line(lines[target_line_idx]) {
        return text.to_string(); // Not a musical line
    }
    
    // Calculate selection relative to the line
    let line_selection_start = selection_start - line_start_pos;
    let line_selection_end = (selection_end - line_start_pos).min(lines[target_line_idx].len());
    
    // Process slur toggling
    toggle_slur_on_line(lines, target_line_idx, line_selection_start, line_selection_end)
}

fn is_musical_line(line: &str) -> bool {
    // Check if line contains musical pitches (2+ pitch characters)
    let tokens = crate::parser::tokenizer::tokenize_with_handwritten_lexer(line);
    let pitch_count = tokens.iter().filter(|t| t.token_type == "PITCH").count();
    pitch_count >= 2
}

fn toggle_slur_on_line(lines: Vec<&str>, target_line: usize, sel_start: usize, sel_end: usize) -> String {
    let mut result_lines = lines.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    
    // Look for existing slur line above target line
    let slur_line_idx = if target_line > 0 {
        // Check if line above has slur characters
        let above_line = &lines[target_line - 1];
        if above_line.contains('_') || above_line.contains('╭') || above_line.contains('─') || above_line.contains('╮') {
            Some(target_line - 1)
        } else {
            None
        }
    } else {
        None
    };
    
    let target_line_str = &lines[target_line];
    
    match slur_line_idx {
        Some(slur_idx) => {
            // Slur line exists - toggle slurs in the selected region
            let mut slur_line = result_lines[slur_idx].chars().collect::<Vec<char>>();
            
            // Ensure slur line is at least as long as target line
            while slur_line.len() < target_line_str.len() {
                slur_line.push(' ');
            }
            
            // Toggle slurs in selection range
            for pos in sel_start..sel_end.min(slur_line.len()) {
                if pos < target_line_str.len() {
                    let target_char = target_line_str.chars().nth(pos).unwrap_or(' ');
                    
                    // Only place slurs above musical characters
                    if is_musical_char(target_char) {
                        if slur_line[pos] == '_' || slur_line[pos] == '─' {
                            slur_line[pos] = ' '; // Remove slur
                        } else {
                            slur_line[pos] = '_'; // Add slur
                        }
                    }
                }
            }
            
            // Clean up slur line - remove if empty
            let slur_line_str: String = slur_line.iter().collect();
            if slur_line_str.trim().is_empty() {
                result_lines.remove(slur_idx);
            } else {
                result_lines[slur_idx] = slur_line_str.trim_end().to_string();
            }
        }
        None => {
            // No slur line exists - create one with slurs for selection
            let mut slur_line = vec![' '; target_line_str.len()];
            
            for pos in sel_start..sel_end.min(slur_line.len()) {
                if pos < target_line_str.len() {
                    let target_char = target_line_str.chars().nth(pos).unwrap_or(' ');
                    
                    if is_musical_char(target_char) {
                        slur_line[pos] = '_';
                    }
                }
            }
            
            let slur_line_str: String = slur_line.iter().collect();
            if !slur_line_str.trim().is_empty() {
                result_lines.insert(target_line, slur_line_str.trim_end().to_string());
            }
        }
    }
    
    result_lines.join("\n")
}

fn is_musical_char(c: char) -> bool {
    // Check if character is a musical pitch
    matches!(c, 
        '1'..='7' |  // Numbers
        'A'..='G' |  // Western
        'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |  // Sargam uppercase
        's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n'     // Sargam lowercase
    )
}

// parse_notation_internal deleted - V2 system used instead




// Initialize function called when WASM module loads
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}