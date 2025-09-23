use crate::parse::{Document, model::{DocumentElement, Stave, StaveLine, ContentLine, NotationSystem}};
use crate::parse::recursive_descent::{parse_document, ParseError};
use crate::parse::content_line_parser_v3::parse_content_line as parse_content_line_v3;
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
use crate::models::pitch_systems::{sargam, western, number};
use crate::models::Degree;
use std::str::FromStr;

/// New pipeline using direct beat parsing (no separate rhythm analysis)
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    // Stage 1: Parse with direct beat creation
    let mut parsed_document = parse_document_with_direct_beats(input)?;

    // Stage 2: Analyze rhythm - add duration information to notes and beats
    crate::rhythm::analyzer::analyze_rhythm_into_document(&mut parsed_document)
        .map_err(|e| format!("Rhythm analysis failed: {}", e))?;

    // Stage 3: Process spatial assignments (octave markers, slurs, syllables)
    let (spatial_document, _spatial_warnings) = process_spatial_assignments_unified(parsed_document.clone())
        .map_err(|e| format!("Spatial assignment failed: {}", e))?;

    // Final document now has rhythm and spatial information
    let document = spatial_document;

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



