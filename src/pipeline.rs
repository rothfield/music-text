use crate::parse::{parse_document, Document};
use crate::stave::{parse_document_staves, ProcessedStave};
use crate::renderers::{render_lilypond, render_vexflow_svg, render_vexflow_data};
use serde::{Deserialize, Serialize};

/// The complete processing pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub original_input: String,
    pub parsed_document: Document,
    pub processed_staves: Vec<ProcessedStave>,
    pub lilypond: String,
    pub vexflow_svg: String,
    pub vexflow_data: serde_json::Value,
}




/// Orchestrates the complete parsing pipeline
/// 
/// Input String → document_parser → stave_parser → converters → ProcessingResult
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    // Stage 1: Parse text into Document structure
    let parsed_document = parse_document(input)?;
    
    // Stage 2: Process document into staves
    let processed_staves = parse_document_staves(parsed_document.clone())?;
    
    // Stage 3: Convert to output formats
    let lilypond = render_lilypond(&processed_staves);
    let vexflow_svg = render_vexflow_svg(&processed_staves);
    let vexflow_data = render_vexflow_data(&processed_staves);
    
    Ok(ProcessingResult {
        original_input: input.to_string(),
        parsed_document,
        processed_staves,
        lilypond,
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