use crate::parse::Document;
use crate::parse::recursive_descent::parse_document;
use crate::spatial::process_spatial_assignments_unified;
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

/// New pipeline using direct beat parsing (no separate rhythm analysis)
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    // Stage 1: Parse with direct beat creation
    let parsed_document = parse_document_with_direct_beats(input)?;

    // Stage 2: Process spatial assignments (octave markers, slurs, syllables) BEFORE rhythm analysis
    let (spatial_document, _spatial_warnings) = process_spatial_assignments_unified(parsed_document.clone())
        .map_err(|e| format!("Spatial assignment failed: {}", e))?;

    // Stage 3: Analyze rhythm - add duration information to notes and beats
    let mut final_document = spatial_document;
    crate::rhythm::analyzer::analyze_rhythm_into_document(&mut final_document)
        .map_err(|e| format!("Rhythm analysis failed: {}", e))?;

    // Final document now has rhythm and spatial information
    let document = final_document;

    // Stage 4: Render from final document
    let lilypond = convert_processed_document_to_lilypond_src(&document, None)?;

    // Stage 5: Render VexFlow from final document
    let vexflow_renderer = VexFlowRenderer::new();
    let vexflow_data = vexflow_renderer.render_data_from_document(&document);
    let vexflow_svg = "".to_string();  // TODO: Implement VexFlow SVG rendering

    Ok(ProcessingResult {
        original_input: input.to_string(),
        document,
        lilypond,
        vexflow_svg,
        vexflow_data,
    })
}

/// Parse document with direct beat creation in content lines
fn parse_document_with_direct_beats(input: &str) -> Result<Document, String> {
    // Direct parsing - no conversion needed
    let document = parse_document(input)?;
    Ok(document)
}



