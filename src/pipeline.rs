use crate::parse::Document;
use crate::renderers::lilypond::renderer::convert_processed_document_to_lilypond_src;
use crate::renderers::vexflow::VexFlowRenderer;
use serde::{Deserialize, Serialize};
/// The complete processing pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub original_input: String,
    pub document: Document,
    pub lilypond: String,
    pub vexflow_svg: String,
    pub vexflow_data: serde_json::Value,
}

/// Simplified pipeline - returns minimal results
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    use crate::models::*;

    // Create document with empty stave
    let document = Document {
        document_uuid: Some(uuid::Uuid::new_v4().to_string()),
        id: uuid::Uuid::new_v4(),
        value: Some(String::new()), // Empty document value
        char_index: 0,
        title: None,
        author: None,
        directives: std::collections::HashMap::new(),
        elements: vec![
            DocumentElement::Stave(Stave {
                id: uuid::Uuid::new_v4(),
                value: Some(String::new()), // Empty stave
                char_index: 0,
                notation_system: NotationSystem::Number,
                line: 1,
                column: 1,
                index_in_doc: 0,
                index_in_line: 0,
                lines: vec![
                    StaveLine::ContentLine(ContentLine {
                        id: uuid::Uuid::new_v4(),
                        value: Some(String::new()), // Empty content line
                        char_index: 0,
                        elements: vec![],
                        consumed_elements: vec![],
                    })
                ],
            })
        ],
        ui_state: UIState::default(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Return minimal processing result
    Ok(ProcessingResult {
        original_input: input.to_string(),
        document,
        lilypond: String::new(), // Empty LilyPond output
        vexflow_svg: String::new(), // Empty VexFlow SVG
        vexflow_data: serde_json::Value::Null, // Null VexFlow data
    })
}




