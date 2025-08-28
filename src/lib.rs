use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Import our existing modules
mod models;
pub mod models_v2; // New type-safe parser AST
mod lexer;
pub mod handwritten_lexer;
mod region_processor_v2; // New region processor for ParsedElement
pub mod rhythm_fsm_v2;
pub mod rhythm_fsm_v2_clean; // New FSM for ParsedElement
mod lyrics_v2; // New lyrics processor for ParsedElement
mod pitch;
mod display;
// mod lilypond_converter; // DELETED - V1 converter unused
pub mod lilypond_converter_v2; // New V2 converter - no conversion needed!
mod lilypond_templates; // Still used by V2 converter
mod outline;
pub mod vexflow_converter_v2; // New V2 VexFlow converter
mod rhythm;
mod notation_detector;
// mod lyrics; // DELETED - replaced with lyrics_v2

pub use models::*;
pub use lexer::lex_text; // tokenize_chunk DELETED - unused
pub use handwritten_lexer::tokenize_with_handwritten_lexer;
// pub use lilypond_converter::{convert_to_lilypond, convert_to_lilypond_with_template}; // DELETED - V1 converter unused
// pub use lilypond_templates::{LilyPondTemplate, TemplateContext}; // DELETED - V1 templates unused
pub use pitch::LilyPondNoteNames;
pub use display::generate_flattened_spatial_view;
pub use outline::{generate_outline, ToOutline};
pub use pitch::{lookup_pitch, Notation}; // guess_notation DELETED - unused
pub use vexflow_converter_v2::{convert_fsm_output_to_vexflow as convert_fsm_output_to_vexflow_v2, VexFlowStave, VexFlowElement, VexFlowAccidental};

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


// unified_parser V1 deleted - use unified_parser_v2 instead

/// Simple conversion from tokens to ParsedElement (replacement for node_builder)
fn convert_tokens_to_parsed_elements(tokens: &[Token], global_notation: crate::pitch::Notation) -> Vec<models_v2::ParsedElement> {
    use crate::models_v2::{ParsedElement, Position};
    use crate::pitch::{lookup_pitch};
    
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

/// V2 Parser using ParsedElement instead of Node
pub fn unified_parser_v2(input_text: &str) -> Result<(models_v2::DocumentV2, String), Box<dyn std::error::Error>> {
    // First detect the notation type
    let detected_notation = notation_detector::detect_notation_type(input_text);
    
    // Use handwritten lexer for tokenization
    let all_tokens = handwritten_lexer::tokenize_with_handwritten_lexer(input_text);
    
    // Create lines_info for spatial analysis compatibility
    let _lines_info = lex_text(input_text);
    
    // Parse metadata and detect notation system
    let (mut metadata, remaining_tokens) = lexer::parse_metadata(&all_tokens);
    metadata.detected_system = Some(detected_notation.as_str().to_string());

    // Convert NotationType to Notation for pitch lookup
    let global_notation = match detected_notation {
        notation_detector::NotationType::Western => crate::pitch::Notation::Western,
        notation_detector::NotationType::Sargam => crate::pitch::Notation::Sargam,
        notation_detector::NotationType::Number => crate::pitch::Notation::Number,
    };

    // Phase 1: Convert tokens to ParsedElement system using global notation detection
    let mut elements = convert_tokens_to_parsed_elements(&remaining_tokens, global_notation);
    
    // Phase 2: Apply slur regions and beat brackets
    region_processor_v2::apply_slurs_and_regions_to_elements(&mut elements, &remaining_tokens);
    
    // Save spatial analysis output for debugging
    let spatial_analysis_yaml = serde_yaml::to_string(&elements).unwrap_or_else(|e| format!("YAML serialization error: {}", e));

    // Phase 3: Group elements into lines and beats using FSM
    let lines_of_music = find_musical_lines(&remaining_tokens);
    
    // Check if we should use the clean FSM
    let use_clean_fsm = std::env::var("USE_CLEAN_FSM").is_ok();
    
    let (fsm_output, mut structured_elements) = if use_clean_fsm {
        eprintln!("Using CLEAN FSM for rhythm processing in lib");
        // Process with clean FSM
        let processed_elements = rhythm_fsm_v2_clean::process_rhythm_v2_clean(elements.clone());
        
        // Convert to OutputItemV2 for compatibility - simplified version for lib
        let mut output = Vec::new();
        for elem in &processed_elements {
            match elem {
                models_v2::ParsedElement::Note { .. } |
                models_v2::ParsedElement::Rest { .. } |
                models_v2::ParsedElement::Dash { .. } => {
                    let beat = rhythm_fsm_v2::BeatV2 {
                        divisions: 1,
                        elements: vec![rhythm_fsm_v2::BeatElement::from(elem.clone()).with_subdivisions(1)],
                        tied_to_previous: false,
                        is_tuplet: false,
                        tuplet_ratio: None,
                    };
                    output.push(rhythm_fsm_v2::OutputItemV2::Beat(beat));
                }
                models_v2::ParsedElement::Barline { .. } => {
                    output.push(rhythm_fsm_v2::OutputItemV2::Barline(elem.value()));
                }
                _ => {}
            }
        }
        (output, processed_elements)
    } else {
        let fsm_output = rhythm_fsm_v2::group_elements_with_fsm_full(&elements, &lines_of_music);
        let structured_elements = rhythm_fsm_v2::convert_fsm_output_to_elements_public(fsm_output.clone());
        (fsm_output, structured_elements)
    };
    
    // Store FSM output for LilyPond conversion
    LAST_FSM_OUTPUT.with(|s| *s.borrow_mut() = fsm_output);
    
    // Phase 4: Process lyrics if present
    if lyrics_v2::has_lyrics(&remaining_tokens, &lines_of_music) {
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


#[wasm_bindgen]
pub fn parse_notation(input_text: &str) -> ParseResult {
    // Set panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    // V1 parse_notation deprecated - use V2 system instead
    match unified_parser_v2(input_text) {
        Ok((document_v2, spatial_analysis_yaml)) => {
            let document: Document = document_v2.into();
            
            // Generate VexFlow output using V2 converter
            let vexflow_json = match convert_fsm_output_to_vexflow_v2(&get_last_fsm_output(), &document.metadata) {
                Ok(staves) => match serde_json::to_string(&staves) {
                    Ok(json) => json,
                    Err(_) => String::new(),
                },
                Err(_) => String::new(),
            };
            
            // Generate LilyPond output using V2 converter
            let lilypond_output = match lilypond_converter_v2::convert_fsm_output_to_lilypond(
                &get_last_fsm_output(),
                &document.metadata,
                crate::pitch::LilyPondNoteNames::English,
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
                vexflow_output: vexflow_json,
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
    static LAST_FSM_OUTPUT: RefCell<Vec<rhythm_fsm_v2::OutputItemV2>> = RefCell::new(Vec::new());
    static LAST_ERROR_MESSAGE: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen]
pub fn get_detected_system() -> String {
    LAST_DETECTED_SYSTEM.with(|s| s.borrow().clone())
}

/// Get the last FSM output for CLI use (avoid running FSM twice)
pub fn get_last_fsm_output() -> Vec<rhythm_fsm_v2::OutputItemV2> {
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

// parse_notation_internal deleted - V2 system used instead




// Initialize function called when WASM module loads
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}