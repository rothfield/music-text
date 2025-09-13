use crate::parse::{parse_document, Document};
use crate::stave::analyze_rhythm;
use crate::renderers::{render_lilypond_from_document, render_vexflow_svg_from_document, render_vexflow_data_from_document};
use serde::{Deserialize, Serialize};

/// The complete processing pipeline output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub original_input: String,
    pub parsed_document: Document,
    pub rhythm_analyzed_document: Document,
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
    
    // Stage 2: Enhance document with rhythm analysis
    let rhythm_analyzed_document = analyze_rhythm(parsed_document.clone())?;
    
    // Stage 3: Render directly from enhanced document
    let lilypond = render_lilypond_from_document(&rhythm_analyzed_document);
    let vexflow_svg = render_vexflow_svg_from_document(&rhythm_analyzed_document);
    let vexflow_data = render_vexflow_data_from_document(&rhythm_analyzed_document);
    
    Ok(ProcessingResult {
        original_input: input.to_string(),
        parsed_document,
        rhythm_analyzed_document,
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
        assert_eq!(result.rhythm_analyzed_document.staves.len(), 1);
    }

    #[test]
    fn test_process_notation_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let result = process_notation(input).unwrap();
        
        assert_eq!(result.parsed_document.staves.len(), 2);
        assert_eq!(result.rhythm_analyzed_document.staves.len(), 2);
    }
}