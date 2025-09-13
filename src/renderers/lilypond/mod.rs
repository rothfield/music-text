pub mod renderer;
pub mod formatters;

use crate::stave::ProcessedStave;
use crate::parse::model::{Directive, Document};
use renderer::LilyPondRenderer;

// Single LilyPond rendering function
pub fn render_lilypond(staves: &[ProcessedStave]) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render(staves)
}

// LilyPond rendering with directives for title/author support
pub fn render_lilypond_with_directives(staves: &[ProcessedStave], directives: &[Directive]) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render_with_directives(staves, directives)
}

// LilyPond rendering directly from Document
pub fn render_lilypond_from_document(document: &Document) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render_from_document(document)
}