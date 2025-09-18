use crate::parse::{parse_document, Document};
use crate::stave_analyzer::analyze_rhythm;
use crate::spatial::process_spatial_assignments;
use crate::renderers::lilypond::render_lilypond_from_document;
use crate::renderers::vexflow::VexFlowRenderer;
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

    // Stage 2: Process spatial assignments (octave markers, slurs, syllables)
    let (spatial_document, _spatial_warnings) = process_spatial_assignments(parsed_document)
        .map_err(|e| format!("Spatial assignment failed: {}", e))?;

    // Stage 3: Analyze rhythm using FSM
    let rhythm_analyzed_document = analyze_rhythm(spatial_document)
        .map_err(|e| format!("Rhythm analysis failed: {}", e))?;

    // Stage 4: Render from rhythm-analyzed document
    let lilypond = render_lilypond_from_document(&rhythm_analyzed_document);

    // Stage 5: Render VexFlow from rhythm-analyzed document
    let vexflow_renderer = VexFlowRenderer::new();
    let vexflow_data = vexflow_renderer.render_data_from_document(&rhythm_analyzed_document);
    let vexflow_svg = "".to_string();  // TODO: Implement VexFlow SVG rendering

    Ok(ProcessingResult {
        original_input: input.to_string(),
        parsed_document: rhythm_analyzed_document.clone(),
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

        // Count staves in elements
        let stave_count = result.parsed_document.elements.iter()
            .filter(|e| matches!(e, crate::parse::model::DocumentElement::Stave(_)))
            .count();
        assert_eq!(stave_count, 1);

        let rhythm_stave_count = result.rhythm_analyzed_document.elements.iter()
            .filter(|e| matches!(e, crate::parse::model::DocumentElement::Stave(_)))
            .count();
        assert_eq!(rhythm_stave_count, 1);
    }

    #[test]
    fn test_process_notation_multi_stave() {
        let input = "|1 2\n\n|3 4";
        let result = process_notation(input).unwrap();

        // Count staves in elements
        let parsed_stave_count = result.parsed_document.elements.iter()
            .filter(|e| matches!(e, crate::parse::model::DocumentElement::Stave(_)))
            .count();
        assert_eq!(parsed_stave_count, 2);

        let rhythm_stave_count = result.rhythm_analyzed_document.elements.iter()
            .filter(|e| matches!(e, crate::parse::model::DocumentElement::Stave(_)))
            .count();
        assert_eq!(rhythm_stave_count, 2);
    }
}