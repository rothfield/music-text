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





