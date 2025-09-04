use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// V2 Parser and Models
pub mod parser_v2;
pub mod parser_v2_pest;
pub mod parser_v2_fsm; // FSM from V1 system
pub mod models;
pub mod converters;

// Re-exports
pub use models::*;
pub use parser_v2::{parse, parse_with_fsm, Document as V2Document, DocumentWithFSM};

/// V2 Parse result structure for WASM API
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ParseResultV2 {
    success: bool,
    error_message: Option<String>,
    document: Option<String>, // JSON serialized V2 document
    vexflow_js: String,
}

#[wasm_bindgen]
impl ParseResultV2 {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }
    
    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn document(&self) -> Option<String> {
        self.document.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn vexflow_js(&self) -> String {
        self.vexflow_js.clone()
    }
}

/// Main WASM parsing function - V2 only
#[wasm_bindgen]
pub fn parse_notation_v2(input_text: &str, notation_system: Option<String>) -> ParseResultV2 {
    match parse_with_fsm(input_text) {
        Ok(document_with_fsm) => {
            // Get FSM elements for VexFlow conversion
            let fsm_elements = &document_with_fsm.staves[0].fsm_output; // TODO: Handle multiple staves
            
            // Convert V2 document to V1 metadata format for compatibility
            let v1_metadata = models::Metadata {
                title: None,
                directives: document_with_fsm.directives.clone().into_iter().map(|(k, v)| {
                    models::Directive { key: k, value: v, row: 0, col: 0 }
                }).collect(),
                detected_system: notation_system,
                attributes: std::collections::HashMap::new(),
            };
            
            // Generate VexFlow JavaScript using existing V1 converter
            let vexflow_js = match converters::vexflow::convert_elements_to_vexflow_js(fsm_elements, &v1_metadata) {
                Ok(js_code) => js_code,
                Err(e) => {
                    return ParseResultV2 {
                        success: false,
                        error_message: Some(format!("VexFlow conversion failed: {}", e)),
                        document: None,
                        vexflow_js: String::new(),
                    }
                }
            };
            
            // Serialize document as JSON
            let document_json = match serde_json::to_string_pretty(&document_with_fsm) {
                Ok(json) => Some(json),
                Err(_) => None,
            };
            
            ParseResultV2 {
                success: true,
                error_message: None,
                document: document_json,
                vexflow_js,
            }
        }
        Err(e) => ParseResultV2 {
            success: false,
            error_message: Some(e.to_string()),
            document: None,
            vexflow_js: String::new(),
        }
    }
}

/// Convert V2 parser result to HTML/CSS format
#[wasm_bindgen]
pub fn convert_v2_to_html_css(input_text: &str, notation_system: Option<String>) -> String {
    let system = notation_system.unwrap_or_else(|| "number".to_string());
    
    match parse(input_text) {
        Ok(document) => {
            let converter = converters::HtmlCssConverterV2::with_system(&system);
            converter.convert_document(&document)
        }
        Err(e) => {
            format!("<div class=\"error\">Parse error: {}</div>", e)
        }
    }
}