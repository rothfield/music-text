pub mod renderer;
pub mod formatters;

use crate::stave::ProcessedStave;
use renderer::LilyPondRenderer;

// Re-export the main functions for rhythm-aware processing
pub fn render_minimal_lilypond(staves: &[ProcessedStave]) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render_minimal(staves)
}

pub fn render_full_lilypond(staves: &[ProcessedStave]) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render_full(staves)
}

pub fn render_web_fast_lilypond(staves: &[ProcessedStave]) -> String {
    let renderer = LilyPondRenderer::new();
    renderer.render_web_fast(staves)
}