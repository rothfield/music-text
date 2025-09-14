use crate::parse::{parse_document, Document};
// use crate::stave::analyze_rhythm;  // TODO: Re-enable when stave module exists
use crate::renderers::lilypond::render_lilypond_from_document;
// use crate::renderers::{render_vexflow_svg_from_document, render_vexflow_data_from_document};  // TODO: Re-enable VexFlow
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

    // Stage 2: For now, just clone the document (TODO: Add rhythm analysis later)
    let rhythm_analyzed_document = parsed_document.clone();

    // Stage 3: Render directly from document
    let lilypond = render_lilypond_from_document(&parsed_document);
    let vexflow_svg = "".to_string();  // TODO: Implement VexFlow SVG rendering
    let vexflow_data = serde_json::json!({
        "input": input,
        "status": "parsed",
        "notes_count": parsed_document.elements.iter()
            .find_map(|element| {
                if let crate::parse::model::DocumentElement::Stave(stave) = element {
                    // Count content elements in the lines
                    stave.lines.iter().find_map(|line| {
                        if let crate::parse::model::StaveLine::Content(elements) = line {
                            Some(elements.len())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            }).unwrap_or(0)
    });

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