use crate::parse::{Document, model::{DocumentElement, Stave, StaveLine, ContentLine, Source, Position, NotationSystem}};
use crate::parse::recursive_descent::{parse_document, ParseError};
use crate::parse::content_line_parser_v3::parse_content_line as parse_content_line_v3;
use crate::spatial::process_spatial_assignments;
use crate::renderers::lilypond::render_lilypond_from_document;
use crate::renderers::vexflow::VexFlowRenderer;
use serde::{Deserialize, Serialize};
use crate::old_pipeline::ProcessingResult;
use crate::models::pitch_systems::{sargam, western, number};
use crate::models::pitch::Degree;
use crate::rhythm::converters::BarlineType;
use std::str::FromStr;

/// New pipeline using direct beat parsing (no separate rhythm analysis)
pub fn process_notation(input: &str) -> Result<ProcessingResult, String> {
    // Stage 1: Parse with direct beat creation
    let mut parsed_document = parse_document_with_direct_beats(input)?;

    // Stage 2: Process spatial assignments (octave markers, slurs, syllables)
    let (spatial_document, _spatial_warnings) = process_spatial_assignments(parsed_document.clone())
        .map_err(|e| format!("Spatial assignment failed: {}", e))?;

    // Stage 3: NO SEPARATE RHYTHM ANALYSIS - already done during parsing!
    // The spatial_document already has beats in ContentLine lines

    // Stage 3: Rhythm analysis already integrated into parsing
    // Beats are now directly in ContentLine.elements as ContentElement::Beat
    let rhythm_analyzed_document = spatial_document;

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

/// Parse document with direct beat creation in content lines
fn parse_document_with_direct_beats(input: &str) -> Result<Document, String> {
    let mut document = parse_document(input)?;

    // Convert Content lines to ContentLine with direct element parsing
    for element in &mut document.elements {
        if let DocumentElement::Stave(stave) = element {
            for (idx, line) in stave.lines.iter_mut().enumerate() {
                if let StaveLine::Content(parsed_elements) = line {
                    // Get the raw content for this line
                    // We need to reconstruct the line text from parsed elements
                    let line_text = reconstruct_line_text(parsed_elements);

                    // Parse directly to ContentLine with beats
                    let content_line = parse_content_line_v3(
                        &line_text,
                        idx + 1,  // Line number (1-based)
                        stave.notation_system,
                        0  // Doc index - would need proper tracking
                    ).map_err(|e| format!("Direct element parsing failed: {}", e))?;

                    // Replace Content with ContentLine
                    *line = StaveLine::ContentLine(content_line);
                }
            }
        }
    }

    Ok(document)
}



/// Reconstruct line text from parsed elements (for re-parsing)
fn reconstruct_line_text(elements: &[crate::rhythm::types::ParsedElement]) -> String {
    use crate::rhythm::types::ParsedElement;

    let mut result = String::new();
    for element in elements {
        match element {
            ParsedElement::Note { value, .. } => result.push_str(value),
            ParsedElement::Barline { style, .. } => result.push_str(style),
            ParsedElement::Whitespace { value, .. } => result.push_str(value),
            ParsedElement::Dash { .. } => result.push('-'),
            ParsedElement::Symbol { value, .. } => result.push_str(value),
            ParsedElement::Newline { value, .. } => result.push_str(value),
            ParsedElement::Rest { value, .. } => result.push_str(value),
            ParsedElement::Unknown { value, .. } => result.push_str(value),
            _ => {} // Skip other elements
        }
    }
    result
}