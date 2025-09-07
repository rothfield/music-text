use crate::document::{parse_document, Document, Stave};
use crate::stave_parser::parse_document_staves;
use crate::renderers::{render_minimal_lilypond, render_full_lilypond, render_vexflow_svg, render_vexflow_data};
use log::warn;
use serde::{Deserialize, Serialize};

/// The complete processing pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub original_input: String,
    pub parsed_document: Document,
    pub processed_staves: Vec<Stave>,
    pub minimal_lilypond: String,
    pub full_lilypond: String,
    pub vexflow_svg: String,
    pub vexflow_data: serde_json::Value,
}

/// Raw PEST parse output (not part of main pipeline but useful for debugging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PestResult {
    pub success: bool,
    pub parse_tree: Option<serde_json::Value>,
    pub error: Option<String>,
}



/// Orchestrates the complete parsing pipeline
/// 
/// Input String → document_parser → stave_parser → converters → ProcessingResult
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    warn!("PIPELINE: Starting process_notation with input: '{}'", input);
    // Stage 1: Parse text into Document structure
    let parsed_document = parse_document(input)?;
    
    // Stage 2: Process document into staves
    let processed_staves = parse_document_staves(parsed_document.clone())?;
    
    // Stage 3: Convert to output formats
    let minimal_lilypond = render_minimal_lilypond(&processed_staves);
    warn!("Generated minimal LilyPond source: {}", minimal_lilypond);
    let full_lilypond = render_full_lilypond(&processed_staves);
    let vexflow_svg = render_vexflow_svg(&processed_staves);
    let vexflow_data = render_vexflow_data(&processed_staves);
    
    Ok(ProcessingResult {
        original_input: input.to_string(),
        parsed_document,
        processed_staves,
        minimal_lilypond,
        full_lilypond,
        vexflow_svg,
        vexflow_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_notation_single_stave() {
        let input = "|1 2 3";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.original_input, input);
        assert_eq!(result.parsed_document.staves.len(), 1);
        assert_eq!(result.processed_staves.len(), 1);
    }

    #[test]
    fn test_process_notation_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.parsed_document.staves.len(), 2);
        assert_eq!(result.processed_staves.len(), 2);
    }
}